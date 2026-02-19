use std::path::Path;

use crate::core::errors::Result;
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
    pub fn encrypt_file(&self, _source: &Path, _dest: &Path) -> Result<()> {
        todo!()
    }

    /// Decrypt a file using the local private key.
    pub fn decrypt_file(&self, _source: &Path, _dest: &Path) -> Result<()> {
        todo!()
    }
}
