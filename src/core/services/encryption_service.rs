use std::path::Path;

use crate::core::errors::{Result, VaulticError};
use crate::core::traits::cipher::CipherBackend;
use crate::core::traits::key_store::KeyStore;

/// Orchestrates encrypt/decrypt operations by combining a
/// `CipherBackend` with a `KeyStore`.
pub struct EncryptionService<C: CipherBackend, K: KeyStore> {
    pub cipher: C,
    pub key_store: K,
}

impl<C: CipherBackend, K: KeyStore> EncryptionService<C, K> {
    /// Encrypt a file for all authorized recipients.
    ///
    /// Reads `source`, encrypts with all keys from the key store,
    /// and writes the ciphertext to `dest`.
    pub fn encrypt_file(&self, source: &Path, dest: &Path) -> Result<()> {
        let plaintext = std::fs::read(source).map_err(|_| VaulticError::FileNotFound {
            path: source.to_path_buf(),
        })?;

        let recipients = self.key_store.list()?;
        if recipients.is_empty() {
            return Err(VaulticError::EncryptionFailed {
                reason: "No recipients configured. Run 'vaultic keys add' first.".into(),
            });
        }

        let ciphertext = self.cipher.encrypt(&plaintext, &recipients)?;

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, ciphertext)?;

        Ok(())
    }

    /// Decrypt a file using the local private key.
    ///
    /// Reads `source` (encrypted), decrypts with the local identity,
    /// and writes the plaintext to `dest`.
    pub fn decrypt_file(&self, source: &Path, dest: &Path) -> Result<()> {
        let plaintext = self.decrypt_to_bytes(source)?;

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, plaintext)?;

        Ok(())
    }

    /// Encrypt raw bytes for all authorized recipients and write to `dest`.
    ///
    /// Avoids writing plaintext to disk — used by `encrypt --all` to
    /// re-encrypt already-decrypted content directly from memory.
    pub fn encrypt_bytes(&self, plaintext: &[u8], dest: &Path) -> Result<()> {
        let recipients = self.key_store.list()?;
        if recipients.is_empty() {
            return Err(VaulticError::EncryptionFailed {
                reason: "No recipients configured. Run 'vaultic keys add' first.".into(),
            });
        }

        let ciphertext = self.cipher.encrypt(plaintext, &recipients)?;

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, ciphertext)?;

        Ok(())
    }

    /// Decrypt a file in memory and return the plaintext bytes.
    ///
    /// Useful for operations that need decrypted content without
    /// writing it to disk (e.g. environment resolution).
    pub fn decrypt_to_bytes(&self, source: &Path) -> Result<Vec<u8>> {
        let ciphertext = std::fs::read(source).map_err(|_| VaulticError::FileNotFound {
            path: source.to_path_buf(),
        })?;

        self.cipher.decrypt(&ciphertext)
    }
}
