use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use rpassword::read_password;
use sha2::{Digest, Sha256};
use std::io::{self, Write};

pub struct CryptoManager {
    master_key: [u8; 32],
}

impl CryptoManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            master_key: derive_master_key().expect("Could not derive master key."),
        })
    }

    #[cfg(test)]
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { master_key: key }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(encrypted_data))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String> {
        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|e| anyhow!("Store possibly corrupt, please recreate your store: {}", e))?;

        if encrypted_bytes.len() < 12 {
            return Err(anyhow!("Invalid encrypted data found."));
        }

        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Error occurred during decryption: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Error occurred during reconstruction: {}", e))
    }
}

fn derive_master_key() -> Result<[u8; 32]> {
    print!("Please enter your master key: ");
    io::stdout().flush().expect("Failed to flush stdout");
    let input = read_password().expect("Failed to read master key");

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt() {
        let crypto = CryptoManager::from_key([0u8; 32]);
        let plaintext = "my_secret_token";

        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_invalid_data() {
        let crypto = CryptoManager::from_key([0u8; 32]);
        let result = crypto.decrypt("not_base64");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_corrupted_data() {
        let crypto = CryptoManager::from_key([0u8; 32]);
        let plaintext = "secret";
        let mut encrypted = crypto.encrypt(plaintext).unwrap();

        // Corrupt one character
        if encrypted.chars().nth(0) != Some('X') {
            encrypted.replace_range(0..1, "X");
        } else {
            encrypted.replace_range(0..1, "Y");
        }
        let result = crypto.decrypt(&encrypted);
        assert!(result.is_err());
    }
}
