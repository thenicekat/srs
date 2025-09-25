use crate::crypto::CryptoManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use dirs;

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

    pub fn delete_token(&mut self, name: String) -> Result<bool> {
        let _ = self.verify_master_key()?;

        let removed = self.database.tokens.remove(&name).is_some();
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
        
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/sh".to_string());
        
        let mut exports = String::new();
        for (name, encrypted_token) in &self.database.tokens {
            let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
            exports.push_str(&format!("export {}={}; ", name, decrypted_token));
        }
        
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("bash");
        
        let mut child = std::process::Command::new(&shell)
            .arg("-c")
            .arg(format!("{} exec {}", exports, shell_name))
            .spawn()?;
        
        child.wait()?;
        Ok(())
    }
}
