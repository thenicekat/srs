use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

pub struct CryptoManager {
    master_key: [u8; 32],
}

impl CryptoManager {
    pub fn new(master_key: &str) -> Result<Self> {
        let key_bytes = master_key.as_bytes();
        let mut key = [0u8; 32];
        key[..32].copy_from_slice(&key_bytes[..32]);
        Ok(Self { master_key: key })
    }
    
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);
        
        Ok(general_purpose::STANDARD.encode(encrypted_data))
    }
    
    pub fn decrypt(&self, encrypted_data: &str) -> Result<String> {
        let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|e| anyhow!("Base64 decode failed: {}", e))?;
        
        if encrypted_bytes.len() < 12 {
            return Err(anyhow!("Invalid encrypted data"));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }
}
