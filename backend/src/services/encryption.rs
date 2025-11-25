use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, rand_core::RngCore};
use base64::{Engine as _, engine::general_purpose};
use std::env;

#[derive(Clone)]
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let key_str = env::var("ENCRYPTION_KEY")
            .or_else(|_| -> Result<String, std::env::VarError> {
                // Generate a warning and use default key for development
                tracing::warn!("ENCRYPTION_KEY not set, using default key for development only");
                Ok("CHANGE_THIS_IN_PRODUCTION_32_BYTES".to_string())
            })?;

        if key_str.len() != 32 {
            return Err("Encryption key must be exactly 32 bytes".into());
        }

        let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext for storage
        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&encrypted_data))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if encrypted_bytes.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("UTF-8 conversion failed: {}", e).into())
    }

    pub fn encrypt_bytes(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        Ok(encrypted_data)
    }

    pub fn decrypt_bytes(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        std::env::set_var("ENCRYPTION_KEY", "test_key_32_bytes_long_exactly!!");
        let service = EncryptionService::new().expect("Failed to create encryption service");
        
        let original = "test password 123!@#";
        let encrypted = service.encrypt(original).expect("Failed to encrypt");
        let decrypted = service.decrypt(&encrypted).expect("Failed to decrypt");
        
        assert_eq!(original, decrypted);
    }
}