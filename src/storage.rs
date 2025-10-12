use crate::crypto::CryptoManager;
use crate::keychain::InMemoryStore;
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
            store: Box::new(KeychainStore::new()?),
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

    fn setup_storage() -> Result<TokenStorage> {
        // Use a constant key to avoid prompting
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        // Choose the store implementation based on platform / CI
        #[cfg(test)]
        let store: Box<dyn SRSStore> = {
            if std::env::var("CI").is_ok() {
                // Use in-memory store in CI
                Box::new(InMemoryStore::new())
            } else {
                // Use real Linux keyring locally
                Box::new(KeychainStore::new()?)
            }
        };

        #[cfg(all(not(test), target_os = "linux"))]
        let store: Box<dyn SRSStore> = Box::new(KeychainStore::new()?);

        #[cfg(all(not(test), target_os = "macos"))]
        let store: Box<dyn SRSStore> = Box::new(KeychainStore::new()?);

        #[cfg(all(not(test), target_os = "windows"))]
        let store: Box<dyn SRSStore> = Box::new(KeychainStore::new()?);

        let storage = TokenStorage {
            store,
            crypto_manager,
        };

        Ok(storage)
    }

    #[test]
    fn store_and_get_token() {
        let mut storage = setup_storage().unwrap();
        storage.store_token("foo", "bar").unwrap();

        let token = storage.get_token("foo").unwrap();
        assert_eq!(token, "bar");
    }

    #[test]
    fn get_nonexistent_token() {
        let storage = setup_storage().unwrap();
        let token = storage.get_token("nonexistent");
        assert!(token.is_err());
    }

    #[test]
    fn delete_token() {
        let mut storage = setup_storage().unwrap();
        storage.store_token("foo", "bar").unwrap();
        let deleted = storage.delete_token("foo").unwrap();
        assert!(deleted);

        let token = storage.get_token("foo");
        assert!(token.is_err());
    }

    #[test]
    fn delete_nonexistent_token() {
        let mut storage = setup_storage().unwrap();
        let result = storage.delete_token("nonexistent").unwrap();
        assert!(!result); // Now returns false instead of error
    }

    #[test]
    fn list_tokens() {
        let mut storage = setup_storage().unwrap();
        storage.store_token("foo", "bar").unwrap();
        storage.store_token("baz", "qux").unwrap();

        let tokens = storage.list_tokens().unwrap();
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"baz".to_string()));
    }

    #[test]
    fn save_and_load() -> Result<()> {
        let mut storage = setup_storage()?;

        // Store a token
        storage.store_token("foo", "bar")?;

        // Create a second instance of storage
        let key = [0u8; 32];
        let crypto_manager = CryptoManager::from_key(key);

        let storage2 = TokenStorage {
            store: Box::new(KeychainStore::new()?),
            crypto_manager,
        };

        // Check if the token is accessible from the second instance
        let token = storage2.get_token("foo")?;
        assert_eq!(token, "bar");

        Ok(())
    }
}
