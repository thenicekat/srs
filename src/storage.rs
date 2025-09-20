use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::crypto::CryptoManager;

#[derive(Serialize, Deserialize)]
struct TokenDatabase {
    tokens: HashMap<String, String>,
}

pub struct TokenStorage {
    file_path: String,
    crypto_manager: CryptoManager,
    database: TokenDatabase,
}

impl TokenStorage {
    pub fn new(file_path: &str, crypto_manager: CryptoManager) -> Result<Self> {
        let mut storage = Self {
            file_path: file_path.to_string(),
            crypto_manager,
            database: TokenDatabase {
                tokens: HashMap::new(),
            },
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
        self.database.tokens.insert(name.to_string(), encrypted_token);
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
        Ok(self.database.tokens.keys().cloned().collect())
    }
    
    pub fn delete_token(&mut self, name: &str) -> Result<bool> {
        let removed = self.database.tokens.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }
}
