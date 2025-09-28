use crate::crypto::CryptoManager;
use anyhow::Result;
use keyring_core::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct TokenDatabase {
    tokens: HashMap<String, String>,
}

pub struct TokenStorage {
    database: TokenDatabase,
    crypto_manager: CryptoManager,
    keyring_entry: Entry,
}

impl TokenStorage {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            use windows_native_keyring_store::Store as WindowsStore;
            let store = WindowsStore::new()?;
            keyring_core::set_default_store(store);
        }

        #[cfg(target_os = "macos")]
        {
            use apple_native_keyring_store::protected::Store as MacOSStore;
            let store = MacOSStore::new()?;
            keyring_core::set_default_store(store);
        }

        #[cfg(target_os = "linux")]
        {
            use dbus_secret_service_keyring_store::Store as LinuxStore;
            let store = LinuxStore::new()?;
            keyring_core::set_default_store(store);
        }

        let crypto_manager: CryptoManager = CryptoManager::new()?;
        let keyring_entry = keyring_core::Entry::new("srs", "thenicekat")?;

        let mut storage = Self {
            database: TokenDatabase {
                tokens: HashMap::new(),
            },
            crypto_manager,
            keyring_entry,
        };

        storage.load()?;
        storage.save()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<()> {
        match self.keyring_entry.get_password() {
            Ok(content) => {
                self.database = serde_json::from_str(&content)?;
            }
            Err(_) => {
                self.database = TokenDatabase {
                    tokens: HashMap::new(),
                };
            }
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.database)?;
        self.keyring_entry.set_password(&content)?;
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

    pub fn populate_tokens_to_child(&self) -> Result<()> {
        let _ = self.verify_master_key()?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // Build environment variables for the child process
        let mut child_env = std::env::vars().collect::<std::collections::HashMap<String, String>>();

        for (name, encrypted_token) in &self.database.tokens {
            let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
            child_env.insert(name.clone(), decrypted_token);
        }

        let mut child = std::process::Command::new(&shell)
            .envs(&child_env)
            .spawn()?;

        child.wait()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_storage() -> TokenStorage {
        #[cfg(target_os = "macos")]
        {
            use apple_native_keyring_store::protected::Store as MacOSStore;
            let store = MacOSStore::new().unwrap();
            keyring_core::set_default_store(store);
        }

        #[cfg(target_os = "windows")]
        {
            use windows_native_keyring_store::Store as WindowsStore;
            let store = WindowsStore::new().unwrap();
            keyring_core::set_default_store(store);
        }

        #[cfg(target_os = "linux")]
        {
            // For Linux tests, we'll use a mock or skip if no store is available
            // This is a simplified approach for testing
        }

        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage = TokenStorage {
            keyring_entry: Entry::new("srs", "thenicekat").unwrap(),
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
        // Create a new instance pointing to the same keyring entry
        // Note: We can't easily delete the keyring entry in tests,
        // but the load method now handles missing passwords gracefully

        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let mut storage2 = TokenStorage {
            keyring_entry: Entry::new("srs", "thenicekat").unwrap(),
            database: TokenDatabase {
                tokens: HashMap::new(),
            },
            crypto_manager,
        };

        storage.store_token("foo", "bar").unwrap();
        storage2.load().unwrap();

        let token = storage2.get_token("foo").unwrap();
        assert_eq!(token.unwrap(), "bar");
    }
}
