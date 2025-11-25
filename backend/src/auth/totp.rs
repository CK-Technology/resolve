use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};

const TOTP_PERIOD: u64 = 30;
const TOTP_DIGITS: usize = 6;

pub fn generate_secret() -> String {
    let mut secret = vec![0u8; 20]; // 160-bit secret
    rand::thread_rng().fill_bytes(&mut secret);
    general_purpose::STANDARD.encode(&secret)
}

pub fn generate_qr_code(email: &str, secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let issuer = "Resolve";
    let account_name = format!("{}:{}", issuer, email);
    let url = format!(
        "otpauth://totp/{}?secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
        urlencoding::encode(&account_name),
        secret,
        urlencoding::encode(issuer),
        TOTP_DIGITS,
        TOTP_PERIOD
    );

    // In a real implementation, you'd generate an actual QR code image
    // For now, return the URL that can be used to generate a QR code
    Ok(url)
}

pub fn verify_totp(secret: &str, code: &str) -> bool {
    let decoded_secret = match general_purpose::STANDARD.decode(secret) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check current window and ±1 window for clock skew tolerance
    for window_offset in [-1, 0, 1] {
        let time_window = (current_time as i64 + (window_offset * TOTP_PERIOD as i64)) as u64 / TOTP_PERIOD;
        let expected_code = generate_totp(&decoded_secret, time_window);
        
        if expected_code == code {
            return true;
        }
    }

    false
}

fn generate_totp(secret: &[u8], time_window: u64) -> String {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

    type HmacSha1 = Hmac<Sha1>;

    let time_bytes = time_window.to_be_bytes();
    
    let mut mac = <HmacSha1 as hmac::Mac>::new_from_slice(secret).unwrap();
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let code = ((result[offset] & 0x7f) as u32) << 24
        | ((result[offset + 1] & 0xff) as u32) << 16
        | ((result[offset + 2] & 0xff) as u32) << 8
        | (result[offset + 3] & 0xff) as u32;

    let otp = code % 10_u32.pow(TOTP_DIGITS as u32);
    format!("{:0width$}", otp, width = TOTP_DIGITS)
}

pub fn encrypt_mfa_secret(secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, secret.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&encrypted))
}

pub fn decrypt_mfa_secret(encrypted_secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted = general_purpose::STANDARD.decode(encrypted_secret)?;
    
    if encrypted.len() < 12 {
        return Err("Invalid encrypted data".into());
    }
    
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(String::from_utf8(plaintext)?)
}

fn get_encryption_key() -> Key<Aes256Gcm> {
    let key_env = std::env::var("MFA_ENCRYPTION_KEY").unwrap_or_else(|_| {
        tracing::warn!("MFA_ENCRYPTION_KEY not set, using default (insecure for production)");
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    });
    
    let key_bytes = hex::decode(&key_env).unwrap_or_else(|_| {
        tracing::error!("Invalid MFA_ENCRYPTION_KEY format, using default");
        vec![0u8; 32]
    });
    
    if key_bytes.len() != 32 {
        tracing::error!("MFA_ENCRYPTION_KEY must be 32 bytes (64 hex chars), using default");
        return *Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        let secret = "JBSWY3DPEHPK3PXP";
        let decoded_secret = general_purpose::STANDARD.decode(secret).unwrap();
        
        // Test with a known time window for reproducible results
        let code = generate_totp(&decoded_secret, 1);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_encryption_decryption() {
        let secret = "test_secret_123";
        let encrypted = encrypt_mfa_secret(secret).unwrap();
        let decrypted = decrypt_mfa_secret(&encrypted).unwrap();
        assert_eq!(secret, decrypted);
    }
}