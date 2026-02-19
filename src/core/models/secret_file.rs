use std::path::PathBuf;

/// A single key-value entry in a secrets file.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line_number: usize,
}

/// Represents a parsed secrets file (e.g. `.env`).
///
/// Preserves ordering and comments so the file can be
/// round-tripped without losing information.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretFile {
    pub entries: Vec<SecretEntry>,
    pub source_path: Option<PathBuf>,
}

impl SecretFile {
    /// Returns the value for the given key, if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
    }

    /// Returns all keys in this file.
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.key.as_str()).collect()
    }
}
