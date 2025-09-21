use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::crypto::CryptoManager;
use std::io::{Write};
use std::sync::LazyLock;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut home_dir = std::env::home_dir().expect("This means something is very wrong.");
    home_dir.push("srs.json");
    home_dir
});

pub static ENV_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("srs.env");
    temp_dir
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
            file_path: (*CONFIG_PATH.clone()).to_path_buf(),
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
        let _ = self.verify_master_key()?;
        Ok(self.database.tokens.keys().cloned().collect())
    }

    fn verify_master_key(&self) -> Result<bool> {
        if self.database.tokens.is_empty() {
            return Err(anyhow::anyhow!("No tokens found, please add a token to start."));
        }
        
        if let Some((_, encrypted_token)) = self.database.tokens.iter().next() {
            Ok(self.crypto_manager.decrypt(encrypted_token).is_ok())
        } else {
            Err(anyhow::anyhow!("Incorrect master key. Cannot delete token."))
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

    pub fn populate_tokens(&self) -> Result<()> {
        let _ = self.verify_master_key()?;
    
        let mut env_file = std::fs::File::create(&*ENV_PATH)?;
        for (name, encrypted_token) in &self.database.tokens {
            let decrypted_token = self.crypto_manager.decrypt(encrypted_token)?;
            writeln!(env_file, "{}={}", name, decrypted_token)?;
        }
        println!("::> Created {} file", (*ENV_PATH).display());
        Ok(())
    }
}
