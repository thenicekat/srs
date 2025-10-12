use crate::crypto::CryptoManager;
use crate::keychain::KeychainStore;
use anyhow::Result;

pub trait SRSStore {
    fn add_token(&self, name: &str, token: &str) -> Result<()>;
    fn get_token(&self, name: &str) -> Result<String>;
    fn list_tokens(&self) -> Result<Vec<String>>;
    fn delete_token(&self, name: &str) -> Result<()>;
}

pub struct TokenStorage {
    store: Box<dyn SRSStore>,
    crypto_manager: CryptoManager,
}

impl TokenStorage {
    pub fn new() -> Result<Self> {
        let crypto_manager: CryptoManager = CryptoManager::new()?;
        let storage = Self {
            store: Box::new(KeychainStore::new()),
            crypto_manager,
        };
        Ok(storage)
    }

    pub fn store_token(&mut self, name: &str, token: &str) -> Result<()> {
        let encrypted_token = self.crypto_manager.encrypt(token)?;
        self.store.add_token(name, &encrypted_token)?;
        Ok(())
    }

    pub fn get_token(&self, name: &str) -> Result<String> {
        match self.store.get_token(name) {
            Ok(encrypted_token) => {
                let decrypted_token = self.crypto_manager.decrypt(&encrypted_token);
                if decrypted_token.is_err() {
                    return Err(anyhow::anyhow!(
                        "Incorrect master key. Cannot decrypt token."
                    ));
                }
                Ok(decrypted_token.unwrap())
            }
            Err(e) => Err(anyhow::anyhow!("Token not found: {}", e)),
        }
    }

    pub fn list_tokens(&self) -> Result<Vec<String>> {
        self.store.list_tokens()
    }

    pub fn delete_token(&mut self, name: &str) -> Result<bool> {
        let token_value = self.store.get_token(name);
        if token_value.is_ok() {
            self.store.delete_token(name)?;
            Ok(true)
        } else {
            println!("::> Token '{}' not found", name);
            Ok(false)
        }
    }

    pub fn populate_tokens_to_child(&self) -> Result<()> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // Build environment variables for the child process
        let mut child_env = std::env::vars().collect::<std::collections::HashMap<String, String>>();

        for name in self.store.list_tokens()? {
            if let Ok(decrypted_token) = self.store.get_token(&name) {
                child_env.insert(name.clone(), decrypted_token);
            }
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
    use uuid::Uuid;
    use crate::keychain::{InMemoryStore, KeychainStore};

    fn setup_storage() -> Result<TokenStorage> {
        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        // Choose the store implementation based on platform / CI
        #[cfg(target_os = "linux")]
        let store: Box<dyn SRSStore> = {
            if std::env::var("CI").is_ok() {
                // Use in-memory store in CI
                Box::new(InMemoryStore::new())
            } else {
                // Use real Linux keyring locally
                Box::new(KeychainStore::new())
            }
        };

        #[cfg(target_os = "macos")]
        let store: Box<dyn SRSStore> = Box::new(KeychainStore::new()?);

        #[cfg(target_os = "windows")]
        let store: Box<dyn SRSStore> = Box::new(KeychainStore::new()?);

        let storage = TokenStorage { store, crypto_manager };
        Ok(storage)
    }

    #[test]
    fn store_and_get_token() -> Result<()> {
        let mut storage = setup_storage()?;
        let name = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();

        storage.store_token(&name, &token)?;
        let retrieved = storage.get_token(&name)?;
        assert_eq!(retrieved, token);

        // Cleanup
        let _ = storage.delete_token(&name)?;
        Ok(())
    }

    #[test]
    fn get_nonexistent_token() -> Result<()> {
        let storage = setup_storage()?;
        let name = Uuid::new_v4().to_string();
        let token = storage.get_token(&name);
        assert!(token.is_err());
        Ok(())
    }

    #[test]
    fn delete_token() -> Result<()> {
        let mut storage = setup_storage()?;
        let name = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();

        storage.store_token(&name, &token)?;
        let deleted = storage.delete_token(&name)?;
        assert!(deleted);

        let retrieved = storage.get_token(&name);
        assert!(retrieved.is_err());
        Ok(())
    }

    #[test]
    fn delete_nonexistent_token() -> Result<()> {
        let mut storage = setup_storage()?;
        let name = Uuid::new_v4().to_string();

        let deleted = storage.delete_token(&name)?;
        assert!(!deleted);
        Ok(())
    }

    #[test]
    fn list_tokens() -> Result<()> {
        let mut storage = setup_storage()?;

        let name1 = Uuid::new_v4().to_string();
        let token1 = Uuid::new_v4().to_string();
        let name2 = Uuid::new_v4().to_string();
        let token2 = Uuid::new_v4().to_string();

        storage.store_token(&name1, &token1)?;
        storage.store_token(&name2, &token2)?;

        let tokens = storage.list_tokens()?;
        assert!(tokens.contains(&name1));
        assert!(tokens.contains(&name2));

        // Cleanup
        let _ = storage.delete_token(&name1)?;
        let _ = storage.delete_token(&name2)?;
        Ok(())
    }

    #[test]
    fn save_and_load() -> Result<()> {
        let mut storage = setup_storage()?;

        let name = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        storage.store_token(&name, &token)?;

        let storage2 = setup_storage()?;
        let retrieved = storage2.get_token(&name)?;
        assert_eq!(retrieved, token);

        // Cleanup
        let _ = storage.delete_token(&name)?;
        Ok(())
    }
}
