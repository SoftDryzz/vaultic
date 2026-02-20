use std::path::{Path, PathBuf};

use crate::core::errors::{Result, VaulticError};
use crate::core::models::key_identity::KeyIdentity;
use crate::core::traits::key_store::KeyStore;

/// File-based key store that persists recipients in a text file.
///
/// Format: one public key per line, with optional `# label` comments.
/// Lines starting with `#` that are NOT inline labels are ignored.
///
/// Example `recipients.txt`:
/// ```text
/// # Added 2026-02-20
/// age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
/// age1x9ynm5k7wz6v3mj8d4qr5tl2hj9nc0kp6w3f7s2y8x4u1v0n3m5q7f2p # dev2
/// ```
#[derive(Clone)]
pub struct FileKeyStore {
    path: PathBuf,
}

impl FileKeyStore {
    /// Create a key store backed by the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Return the file path this store reads from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Parse a single line into a `KeyIdentity`, if it contains a key.
    fn parse_line(line: &str) -> Option<KeyIdentity> {
        let trimmed = line.trim();

        // Skip empty lines and pure comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        // Split key from optional inline label: "age1... # label"
        let (key, label) = match trimmed.split_once('#') {
            Some((k, l)) => (k.trim().to_string(), Some(l.trim().to_string())),
            None => (trimmed.to_string(), None),
        };

        if key.is_empty() {
            return None;
        }

        Some(KeyIdentity {
            public_key: key,
            label,
            added_at: None,
        })
    }

    /// Serialize all identities back to the file format.
    fn serialize(identities: &[KeyIdentity]) -> String {
        identities
            .iter()
            .map(|ki| match &ki.label {
                Some(label) => format!("{} # {}", ki.public_key, label),
                None => ki.public_key.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}

impl KeyStore for FileKeyStore {
    fn add(&self, identity: &KeyIdentity) -> Result<()> {
        let mut existing = self.list()?;

        // Check for duplicates
        if existing
            .iter()
            .any(|ki| ki.public_key == identity.public_key)
        {
            return Err(VaulticError::KeyAlreadyExists {
                identity: identity.public_key.clone(),
            });
        }

        existing.push(identity.clone());
        std::fs::write(&self.path, Self::serialize(&existing))?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<KeyIdentity>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content =
            std::fs::read_to_string(&self.path).map_err(|_| VaulticError::FileNotFound {
                path: self.path.clone(),
            })?;

        Ok(content.lines().filter_map(Self::parse_line).collect())
    }

    fn remove(&self, public_key: &str) -> Result<()> {
        let existing = self.list()?;

        if !existing.iter().any(|ki| ki.public_key == public_key) {
            return Err(VaulticError::KeyNotFound {
                identity: public_key.to_string(),
            });
        }

        let filtered: Vec<_> = existing
            .into_iter()
            .filter(|ki| ki.public_key != public_key)
            .collect();

        std::fs::write(&self.path, Self::serialize(&filtered))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, FileKeyStore) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("recipients.txt");
        let store = FileKeyStore::new(path);
        (dir, store)
    }

    fn sample_key(suffix: &str) -> KeyIdentity {
        KeyIdentity {
            public_key: format!("age1testkey{suffix}"),
            label: None,
            added_at: None,
        }
    }

    #[test]
    fn list_empty_file_returns_empty() {
        let (_dir, store) = temp_store();
        let keys = store.list().unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn add_and_list() {
        let (_dir, store) = temp_store();
        let key = sample_key("abc");

        store.add(&key).unwrap();

        let keys = store.list().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].public_key, "age1testkeyabc");
    }

    #[test]
    fn add_with_label() {
        let (_dir, store) = temp_store();
        let key = KeyIdentity {
            public_key: "age1testkey123".into(),
            label: Some("cristo".into()),
            added_at: None,
        };

        store.add(&key).unwrap();

        let keys = store.list().unwrap();
        assert_eq!(keys[0].label.as_deref(), Some("cristo"));
    }

    #[test]
    fn add_duplicate_fails() {
        let (_dir, store) = temp_store();
        let key = sample_key("dup");

        store.add(&key).unwrap();
        let result = store.add(&key);
        assert!(result.is_err());
    }

    #[test]
    fn remove_existing_key() {
        let (_dir, store) = temp_store();
        let key1 = sample_key("one");
        let key2 = sample_key("two");

        store.add(&key1).unwrap();
        store.add(&key2).unwrap();
        store.remove("age1testkeyone").unwrap();

        let keys = store.list().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].public_key, "age1testkeytwo");
    }

    #[test]
    fn remove_nonexistent_fails() {
        let (_dir, store) = temp_store();
        let result = store.remove("age1doesnotexist");
        assert!(result.is_err());
    }

    #[test]
    fn parse_line_with_label() {
        let ki = FileKeyStore::parse_line("age1abc123 # dev-team").unwrap();
        assert_eq!(ki.public_key, "age1abc123");
        assert_eq!(ki.label.as_deref(), Some("dev-team"));
    }

    #[test]
    fn parse_line_skips_comments() {
        assert!(FileKeyStore::parse_line("# this is a comment").is_none());
        assert!(FileKeyStore::parse_line("").is_none());
        assert!(FileKeyStore::parse_line("  ").is_none());
    }
}
