use crate::core::errors::Result;
use crate::core::models::key_identity::KeyIdentity;

/// Port for managing authorized recipients (public keys).
pub trait KeyStore: Send + Sync {
    /// Add a recipient to the store.
    fn add(&self, identity: &KeyIdentity) -> Result<()>;

    /// List all authorized recipients.
    fn list(&self) -> Result<Vec<KeyIdentity>>;

    /// Remove a recipient by its public key string.
    fn remove(&self, public_key: &str) -> Result<()>;
}
