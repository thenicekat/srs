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
