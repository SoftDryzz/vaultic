use crate::core::errors::Result;
use crate::core::models::key_identity::KeyIdentity;
use crate::core::traits::key_store::KeyStore;

/// Manages recipient keys through a `KeyStore` backend.
pub struct KeyService<K: KeyStore> {
    pub store: K,
}

impl<K: KeyStore> KeyService<K> {
    /// Add a new recipient.
    pub fn add_key(&self, identity: &KeyIdentity) -> Result<()> {
        self.store.add(identity)
    }

    /// List all authorized recipients.
    pub fn list_keys(&self) -> Result<Vec<KeyIdentity>> {
        self.store.list()
    }

    /// Remove a recipient by public key.
    pub fn remove_key(&self, public_key: &str) -> Result<()> {
        self.store.remove(public_key)
    }
}
