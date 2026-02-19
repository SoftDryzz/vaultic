use crate::core::errors::Result;
use crate::core::models::key_identity::KeyIdentity;

/// Port for encryption/decryption backends.
///
/// Implementations live in `adapters::cipher` (e.g. AgeBackend, GpgBackend).
/// The core layer only depends on this trait, never on a concrete backend.
pub trait CipherBackend: Send + Sync {
    /// Encrypt plaintext for the given recipients.
    fn encrypt(&self, plaintext: &[u8], recipients: &[KeyIdentity]) -> Result<Vec<u8>>;

    /// Decrypt ciphertext using the local private key.
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;

    /// Human-readable name of this backend (e.g. "age", "gpg").
    fn name(&self) -> &str;
}
