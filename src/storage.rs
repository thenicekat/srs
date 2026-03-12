use crate::crypto::CryptoManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data_local_dir = dirs::data_local_dir().unwrap();
    data_local_dir.push("srs");
    let _ = fs::create_dir_all(&data_local_dir);
    data_local_dir.push("srs.json");
    data_local_dir
});

#[derive(Serialize, Deserialize)]
struct TokenDatabase {
    tokens: HashMap<String, String>,
    #[serde(default)]
    aliases: HashMap<String, String>,
}

pub struct TokenStorage {
    file_path: PathBuf,
    database: TokenDatabase,
    crypto_manager: CryptoManager,
}

impl TokenStorage {
    pub fn new() -> Result<Self> {
        let crypto_manager: CryptoManager = CryptoManager::new()?;
        let mut storage = Self {
            file_path: CONFIG_PATH.to_path_buf(),
            database: TokenDatabase {
                tokens: HashMap::new(),
                aliases: HashMap::new(),
            },
            crypto_manager,
        };

        storage.load()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<()> {
        if Path::new(&self.file_path).exists() {
            let content = fs::read_to_string(&self.file_path)?;
            self.database = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.database)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn store_token(&mut self, name: &str, token: &str) -> Result<()> {
        let encrypted_token = self.crypto_manager.encrypt(token)?;
        self.database
            .tokens
            .insert(name.to_string(), encrypted_token);
        self.save()?;
        Ok(())
    }

    pub fn get_token(&self, name: &str) -> Result<Option<String>> {
        let actual_name = self.resolve_alias(name);
        match self.database.tokens.get(actual_name) {
            Some(encrypted_token) => {
                let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
                Ok(Some(decrypted_token))
            }
            None => Ok(None),
        }
    }

    fn resolve_alias<'a>(&'a self, name: &'a str) -> &'a str {
        self.database
            .aliases
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or(name)
    }

    pub fn list_tokens(&self) -> Result<Vec<String>> {
        let _ = self.verify_master_key()?;
        Ok(self.database.tokens.keys().cloned().collect())
    }

    fn verify_master_key(&self) -> Result<bool> {
        if self.database.tokens.is_empty() {
            return Err(anyhow::anyhow!(
                "No tokens found, please add a token to start."
            ));
        }

        if let Some((_, encrypted_token)) = self.database.tokens.iter().next() {
            Ok(self.crypto_manager.decrypt(encrypted_token).is_ok())
        } else {
            Err(anyhow::anyhow!(
                "Incorrect master key. Cannot delete token."
            ))
        }
    }

    pub fn delete_token(&mut self, name: &str) -> Result<bool> {
        let _ = self.verify_master_key()?;

        let removed = self.database.tokens.remove(name).is_some();
        if removed {
            self.database.aliases.retain(|_, target| target != name);
            self.save()?;
            println!("::> Token '{}' deleted successfully!", name);
        } else {
            println!("::> Token '{}' not found", name);
        }
        Ok(removed)
    }

    pub fn populate_tokens_to_child(&self) -> Result<()> {
        let _ = self.verify_master_key()?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let mut child_env = std::env::vars().collect::<HashMap<String, String>>();

        for (name, encrypted_token) in &self.database.tokens {
            let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
            child_env.insert(name.clone(), decrypted_token);
        }

        for (alias, target) in &self.database.aliases {
            if let Some(value) = child_env.get(target) {
                child_env.insert(alias.clone(), value.clone());
            }
        }

        let mut child = std::process::Command::new(&shell)
            .envs(&child_env)
            .spawn()?;

        child.wait()?;
        Ok(())
    }

    pub fn add_alias(&mut self, alias: &str, target: &str) -> Result<()> {
        let _ = self.verify_master_key()?;

        if !self.database.tokens.contains_key(target) {
            return Err(anyhow::anyhow!("Target token '{}' does not exist", target));
        }

        if self.database.tokens.contains_key(alias) {
            return Err(anyhow::anyhow!(
                "'{}' already exists as a token name",
                alias
            ));
        }

        if self.database.aliases.contains_key(alias) {
            return Err(anyhow::anyhow!("Alias '{}' already exists", alias));
        }

        self.database
            .aliases
            .insert(alias.to_string(), target.to_string());
        self.save()?;
        Ok(())
    }

    pub fn remove_alias(&mut self, alias: &str) -> Result<bool> {
        let _ = self.verify_master_key()?;

        let removed = self.database.aliases.remove(alias).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn list_aliases(&self) -> Result<Vec<(String, String)>> {
        if !self.database.tokens.is_empty() {
            let _ = self.verify_master_key()?;
        }
        Ok(self
            .database
            .aliases
            .iter()
            .map(|(alias, target)| (alias.clone(), target.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn setup_storage() -> TokenStorage {
        let temp_path = std::env::temp_dir().join(format!("srs_test_{}.json", Uuid::new_v4()));
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }

        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage = TokenStorage {
            file_path: temp_path,
            database: TokenDatabase {
                tokens: HashMap::new(),
                aliases: HashMap::new(),
            },
            crypto_manager,
        };

        storage.load().unwrap();
        storage
    }

    #[test]
    fn store_and_get_token() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();

        let token = storage.get_token("foo").unwrap();
        assert_eq!(token.unwrap(), "bar");
    }

    #[test]
    fn get_nonexistent_token() {
        let storage = setup_storage();
        let token = storage.get_token("nonexistent").unwrap();
        assert!(token.is_none());
    }

    #[test]
    fn delete_token() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();
        let deleted = storage.delete_token("foo").unwrap();
        assert!(deleted);

        let token = storage.get_token("foo").unwrap();
        assert!(token.is_none());
    }

    #[test]
    fn delete_nonexistent_token() {
        let mut storage = setup_storage();
        let result = storage.delete_token("nonexistent").is_err();
        assert!(result);
    }

    #[test]
    fn list_tokens() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();
        storage.store_token("baz", "qux").unwrap();

        let tokens = storage.list_tokens().unwrap();
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"baz".to_string()));
    }

    #[test]
    fn verify_master_key_with_tokens() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();
        let verified = storage.verify_master_key().unwrap();
        assert!(verified);
    }

    #[test]
    fn save_and_load() {
        let mut storage = setup_storage();
        let temp_path = &storage.file_path;
        if temp_path.exists() {
            let _ = std::fs::remove_file(temp_path);
        }

        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage2 = TokenStorage {
            file_path: temp_path.to_path_buf(),
            database: TokenDatabase {
                tokens: HashMap::new(),
                aliases: HashMap::new(),
            },
            crypto_manager,
        };

        storage.store_token("foo", "bar").unwrap();
        storage2.load().unwrap();

        let token = storage2.get_token("foo").unwrap();
        assert_eq!(token.unwrap(), "bar");
    }

    #[test]
    fn add_and_get_alias() {
        let mut storage = setup_storage();
        storage.store_token("MY_TOKEN", "secret_value").unwrap();
        storage.add_alias("MY_ALIAS", "MY_TOKEN").unwrap();

        let token = storage.get_token("MY_ALIAS").unwrap();
        assert_eq!(token.unwrap(), "secret_value");
    }

    #[test]
    fn add_alias_to_nonexistent_token() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();
        let result = storage.add_alias("alias", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_with_existing_token_name() {
        let mut storage = setup_storage();
        storage.store_token("foo", "bar").unwrap();
        storage.store_token("baz", "qux").unwrap();
        let result = storage.add_alias("foo", "baz");
        assert!(result.is_err());
    }

    #[test]
    fn remove_alias() {
        let mut storage = setup_storage();
        storage.store_token("MY_TOKEN", "secret_value").unwrap();
        storage.add_alias("MY_ALIAS", "MY_TOKEN").unwrap();

        let removed = storage.remove_alias("MY_ALIAS").unwrap();
        assert!(removed);

        let token = storage.get_token("MY_ALIAS").unwrap();
        assert!(token.is_none());
    }

    #[test]
    fn list_aliases() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN1", "value1").unwrap();
        storage.store_token("TOKEN2", "value2").unwrap();
        storage.add_alias("ALIAS1", "TOKEN1").unwrap();
        storage.add_alias("ALIAS2", "TOKEN1").unwrap();
        storage.add_alias("ALIAS3", "TOKEN2").unwrap();

        let aliases = storage.list_aliases().unwrap();
        assert_eq!(aliases.len(), 3);
        assert!(aliases.contains(&("ALIAS1".to_string(), "TOKEN1".to_string())));
        assert!(aliases.contains(&("ALIAS2".to_string(), "TOKEN1".to_string())));
        assert!(aliases.contains(&("ALIAS3".to_string(), "TOKEN2".to_string())));
    }

    #[test]
    fn delete_token_removes_aliases() {
        let mut storage = setup_storage();
        storage.store_token("MY_TOKEN", "secret_value").unwrap();
        storage.add_alias("ALIAS1", "MY_TOKEN").unwrap();
        storage.add_alias("ALIAS2", "MY_TOKEN").unwrap();

        storage.delete_token("MY_TOKEN").unwrap();

        let aliases = storage.list_aliases().unwrap();
        assert_eq!(aliases.len(), 0);
    }

    #[test]
    fn cannot_alias_to_alias() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN", "value").unwrap();
        storage.add_alias("ALIAS1", "TOKEN").unwrap();

        let result = storage.add_alias("ALIAS2", "ALIAS1");
        assert!(result.is_err());
    }

    #[test]
    fn cannot_overwrite_existing_alias() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN1", "value1").unwrap();
        storage.store_token("TOKEN2", "value2").unwrap();
        storage.add_alias("MY_ALIAS", "TOKEN1").unwrap();

        let result = storage.add_alias("MY_ALIAS", "TOKEN2");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_aliases_for_same_token() {
        let mut storage = setup_storage();
        storage
            .store_token("GITHUB_TOKEN", "ghp_secret123")
            .unwrap();
        storage.add_alias("GH_TOKEN", "GITHUB_TOKEN").unwrap();
        storage.add_alias("GITHUB_PAT", "GITHUB_TOKEN").unwrap();
        storage.add_alias("GH_PAT", "GITHUB_TOKEN").unwrap();

        assert_eq!(
            storage.get_token("GITHUB_TOKEN").unwrap().unwrap(),
            "ghp_secret123"
        );
        assert_eq!(
            storage.get_token("GH_TOKEN").unwrap().unwrap(),
            "ghp_secret123"
        );
        assert_eq!(
            storage.get_token("GITHUB_PAT").unwrap().unwrap(),
            "ghp_secret123"
        );
        assert_eq!(
            storage.get_token("GH_PAT").unwrap().unwrap(),
            "ghp_secret123"
        );

        let aliases = storage.list_aliases().unwrap();
        assert_eq!(aliases.len(), 3);
    }

    #[test]
    fn remove_nonexistent_alias() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN", "value").unwrap();

        let removed = storage.remove_alias("NONEXISTENT").unwrap();
        assert!(!removed);
    }

    #[test]
    fn update_token_value_aliases_still_work() {
        let mut storage = setup_storage();
        storage.store_token("API_KEY", "old_value").unwrap();
        storage.add_alias("KEY", "API_KEY").unwrap();

        assert_eq!(storage.get_token("KEY").unwrap().unwrap(), "old_value");

        storage.store_token("API_KEY", "new_value").unwrap();

        assert_eq!(storage.get_token("API_KEY").unwrap().unwrap(), "new_value");
        assert_eq!(storage.get_token("KEY").unwrap().unwrap(), "new_value");
    }

    #[test]
    fn list_tokens_excludes_aliases() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN1", "value1").unwrap();
        storage.store_token("TOKEN2", "value2").unwrap();
        storage.add_alias("ALIAS1", "TOKEN1").unwrap();
        storage.add_alias("ALIAS2", "TOKEN2").unwrap();

        let tokens = storage.list_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&"TOKEN1".to_string()));
        assert!(tokens.contains(&"TOKEN2".to_string()));
        assert!(!tokens.contains(&"ALIAS1".to_string()));
        assert!(!tokens.contains(&"ALIAS2".to_string()));
    }

    #[test]
    fn delete_token_with_no_aliases() {
        let mut storage = setup_storage();
        storage.store_token("SOLO_TOKEN", "value").unwrap();

        let deleted = storage.delete_token("SOLO_TOKEN").unwrap();
        assert!(deleted);

        assert!(storage.get_token("SOLO_TOKEN").unwrap().is_none());
        assert_eq!(storage.list_aliases().unwrap().len(), 0);
    }

    #[test]
    fn save_and_load_preserves_aliases() {
        let mut storage = setup_storage();
        let temp_path = storage.file_path.clone();

        storage.store_token("TOKEN", "secret").unwrap();
        storage.add_alias("ALIAS1", "TOKEN").unwrap();
        storage.add_alias("ALIAS2", "TOKEN").unwrap();

        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);
        let mut storage2 = TokenStorage {
            file_path: temp_path,
            database: TokenDatabase {
                tokens: HashMap::new(),
                aliases: HashMap::new(),
            },
            crypto_manager,
        };
        storage2.load().unwrap();

        assert_eq!(storage2.get_token("TOKEN").unwrap().unwrap(), "secret");
        assert_eq!(storage2.get_token("ALIAS1").unwrap().unwrap(), "secret");
        assert_eq!(storage2.get_token("ALIAS2").unwrap().unwrap(), "secret");

        let aliases = storage2.list_aliases().unwrap();
        assert_eq!(aliases.len(), 2);
    }

    #[test]
    fn empty_storage_list_operations() {
        let storage = setup_storage();

        let result = storage.list_tokens();
        assert!(result.is_err());

        let result = storage.list_aliases();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn special_characters_in_names() {
        let mut storage = setup_storage();
        storage.store_token("MY_TOKEN-123", "value").unwrap();
        storage.add_alias("MY.ALIAS_456", "MY_TOKEN-123").unwrap();

        assert_eq!(storage.get_token("MY_TOKEN-123").unwrap().unwrap(), "value");
        assert_eq!(storage.get_token("MY.ALIAS_456").unwrap().unwrap(), "value");
    }

    #[test]
    fn case_sensitive_names() {
        let mut storage = setup_storage();
        storage.store_token("token", "lowercase").unwrap();
        storage.store_token("TOKEN", "uppercase").unwrap();

        assert_eq!(storage.get_token("token").unwrap().unwrap(), "lowercase");
        assert_eq!(storage.get_token("TOKEN").unwrap().unwrap(), "uppercase");
        assert!(storage.get_token("Token").unwrap().is_none());
    }

    #[test]
    fn long_token_values() {
        let mut storage = setup_storage();
        let long_value = "a".repeat(10000);

        storage.store_token("LONG_TOKEN", &long_value).unwrap();
        storage.add_alias("LONG_ALIAS", "LONG_TOKEN").unwrap();

        assert_eq!(
            storage.get_token("LONG_TOKEN").unwrap().unwrap(),
            long_value
        );
        assert_eq!(
            storage.get_token("LONG_ALIAS").unwrap().unwrap(),
            long_value
        );
    }

    #[test]
    fn unicode_in_values() {
        let mut storage = setup_storage();
        let unicode_value = "Hello 世界 🌍 مرحبا";

        storage.store_token("UNICODE_TOKEN", unicode_value).unwrap();
        storage.add_alias("UNICODE_ALIAS", "UNICODE_TOKEN").unwrap();

        assert_eq!(
            storage.get_token("UNICODE_TOKEN").unwrap().unwrap(),
            unicode_value
        );
        assert_eq!(
            storage.get_token("UNICODE_ALIAS").unwrap().unwrap(),
            unicode_value
        );
    }

    #[test]
    fn delete_token_then_recreate_with_same_name() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN", "value1").unwrap();
        storage.add_alias("ALIAS", "TOKEN").unwrap();

        storage.delete_token("TOKEN").unwrap();
        assert!(storage.get_token("TOKEN").unwrap().is_none());
        assert!(storage.get_token("ALIAS").unwrap().is_none());

        storage.store_token("TOKEN", "value2").unwrap();
        assert_eq!(storage.get_token("TOKEN").unwrap().unwrap(), "value2");

        assert!(storage.get_token("ALIAS").unwrap().is_none());
    }

    #[test]
    fn alias_chain_prevention() {
        let mut storage = setup_storage();
        storage.store_token("TOKEN", "value").unwrap();
        storage.add_alias("ALIAS1", "TOKEN").unwrap();

        let result = storage.add_alias("ALIAS2", "ALIAS1");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
