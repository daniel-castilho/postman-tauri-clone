use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;

pub struct EncryptionManager {
    key: [u8; 32],
}

impl EncryptionManager {
    /// Creates a new EncryptionManager deriving a 256-bit key from the master password
    /// using Argon2id, the recommended algorithm for password hashing.
    /// The salt should be unique per installation and stored securely.
    pub fn new(master_password: &str, salt_b64: &str) -> Result<Self, String> {
        // Parse salt from base64
        let salt = SaltString::from_b64(salt_b64)
            .map_err(|e| format!("Invalid salt: {}", e))?;

        // Use Argon2id with OWASP recommended parameters (m=65536, t=3, p=4)
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(65536, 3, 4, Some(32))
                .map_err(|e| format!("Invalid Argon2 params: {}", e))?,
        );

        // Hash password and extract key bytes
        let password_hash = argon2
            .hash_password(master_password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        // Extract the raw hash output for AES key
        let hash_output = password_hash.hash
            .ok_or_else(|| "Failed to derive key: no hash output from Argon2".to_string())?;
        let hash_bytes = hash_output.as_bytes();

        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);

        Ok(Self { key })
    }

    /// Generates a new random salt for key derivation.
    /// This should be stored securely and reused for subsequent encryption operations
    /// with the same master password.
    // Consumed by the upcoming vault adapter (see AGENTS.md technical debt);
    // not yet wired into any IPC command.
    #[allow(dead_code)]
    pub fn generate_salt() -> String {
        SaltString::generate(&mut rand::rngs::OsRng).to_string()
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| e.to_string())?;
        
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| e.to_string())?;

        // Format: [NONCE(12 bytes)][CIPHERTEXT]
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);
        
        Ok(STANDARD.encode(combined))
    }

    pub fn decrypt(&self, encrypted_b64: &str) -> Result<String, String> {
        let combined = STANDARD.decode(encrypted_b64).map_err(|e| e.to_string())?;
        if combined.len() < 12 {
            return Err("Invalid encrypted payload".into());
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| e.to_string())?;
        
        let decrypted = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        String::from_utf8(decrypted).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_salt_produces_a_usable_manager() {
        let salt = EncryptionManager::generate_salt();
        assert!(!salt.is_empty());
        // The generated salt must be accepted by the key derivation.
        assert!(EncryptionManager::new("password", &salt).is_ok());
    }

    #[test]
    fn encrypt_decrypt_roundtrips_plaintext() {
        let salt = EncryptionManager::generate_salt();
        let manager = EncryptionManager::new("correct horse battery staple", &salt)
            .expect("manager");

        let ciphertext = manager.encrypt("vault secret").expect("encrypt");
        assert_ne!(ciphertext, "vault secret");

        let plaintext = manager.decrypt(&ciphertext).expect("decrypt");
        assert_eq!(plaintext, "vault secret");
    }

    #[test]
    fn decrypt_rejects_garbage_payloads() {
        let salt = EncryptionManager::generate_salt();
        let manager = EncryptionManager::new("password", &salt).expect("manager");
        assert!(manager.decrypt("not-base64!!").is_err());
        assert!(manager.decrypt("QUJD").is_err(), "short payload must fail");
    }

    #[test]
    fn ciphertext_differs_between_runs_due_to_random_nonce() {
        let salt = EncryptionManager::generate_salt();
        let manager = EncryptionManager::new("password", &salt).expect("manager");
        let first = manager.encrypt("same input").expect("encrypt");
        let second = manager.encrypt("same input").expect("encrypt");
        assert_ne!(first, second);
    }
}
