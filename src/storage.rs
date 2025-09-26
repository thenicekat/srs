use crate::crypto::CryptoManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
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

pub static ENV_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data_local_dir = dirs::data_local_dir().unwrap();
    data_local_dir.push("srs");
    let _ = fs::create_dir_all(&data_local_dir);
    data_local_dir.push("__srs__.env");
    data_local_dir
});

#[derive(Serialize, Deserialize)]
struct TokenDatabase {
    tokens: HashMap<String, String>,
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
        match self.database.tokens.get(name) {
            Some(encrypted_token) => {
                let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
                Ok(Some(decrypted_token))
            }
            None => Ok(None),
        }
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
            self.save()?;
            println!("::> Token '{}' deleted successfully!", name);
        } else {
            println!("::> Token '{}' not found", name);
        }
        Ok(removed)
    }

    pub fn populate_tokens(&self) -> Result<()> {
        let _ = self.verify_master_key()?;

        let mut env_file = std::fs::File::create(&*ENV_PATH)?;
        for (name, encrypted_token) in &self.database.tokens {
            let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
            writeln!(env_file, "{}={}", name, decrypted_token)?;
        }
        Ok(())
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

        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage = TokenStorage {
            file_path: temp_path,
            database: TokenDatabase {
                tokens: HashMap::new(),
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
        // Create a new instance pointing to the same file
        let temp_path = &storage.file_path;
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }

        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage2 = TokenStorage {
            file_path: temp_path.to_path_buf(),
            database: TokenDatabase {
                tokens: HashMap::new(),
            },
            crypto_manager,
        };

        storage2.load().unwrap();
        storage.store_token("foo", "bar").unwrap();

        let token = storage2.get_token("foo").unwrap();
        assert_eq!(token.unwrap(), "bar");
    }
}
