use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::errors::ChainDBError;

#[derive(Clone, Serialize)]
pub struct DataEncryption {
    #[serde(skip)]
    cipher: Aes256Gcm,
    master_key: Vec<u8>,
}

impl Default for DataEncryption {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Debug for DataEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataEncryption")
            .field("master_key", &self.master_key)
            .finish()
    }
}

impl DataEncryption {
    pub fn new(password: &str) -> Self {
        // Derive a 32-byte key from the password using SHA-256
        let mut hasher = Sha256::default();
        hasher.update(password.as_bytes());
        let master_key = hasher.finalize().to_vec();

        // Create cipher with master key
        let key = Key::<Aes256Gcm>::from_slice(&master_key);
        let cipher = Aes256Gcm::new(key);

        Self { cipher, master_key }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, ChainDBError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_data = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| ChainDBError::EncryptionError(e.to_string()))?;

        let mut result = Vec::with_capacity(12 + encrypted_data.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&encrypted_data);

        Ok(result)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, ChainDBError> {
        if encrypted_data.len() < 12 {
            return Err(ChainDBError::DecryptionError(
                "Invalid encrypted data length".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let decrypted_data = self
            .cipher
            .decrypt(nonce, &encrypted_data[12..])
            .map_err(|e| ChainDBError::DecryptionError(e.to_string()))?;

        Ok(decrypted_data)
    }
}
