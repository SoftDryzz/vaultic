use std::path::PathBuf;

/// A single key-value entry in a secrets file.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line_number: usize,
}

/// Represents any line in a secrets file.
///
/// This enum allows preserving the exact structure of the original
/// file — comments, blank lines, and variable entries — so the
/// file can be round-tripped without losing formatting.
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    /// A key-value variable entry.
    Entry(SecretEntry),
    /// A comment line (e.g. `# Database config`).
    Comment(String),
    /// An empty or whitespace-only line.
    Blank,
}

/// Represents a parsed secrets file (e.g. `.env`).
///
/// Preserves ordering, comments, and blank lines so the file can be
/// round-tripped without losing information.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretFile {
    pub lines: Vec<Line>,
    pub source_path: Option<PathBuf>,
}

impl SecretFile {
    /// Returns the value for the given key, if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
    }

    /// Returns all keys in this file.
    pub fn keys(&self) -> Vec<&str> {
        self.entries().map(|e| e.key.as_str()).collect()
    }

    /// Iterates over only the key-value entries, skipping comments and blanks.
    pub fn entries(&self) -> impl Iterator<Item = &SecretEntry> {
        self.lines.iter().filter_map(|line| match line {
            Line::Entry(entry) => Some(entry),
            _ => None,
        })
    }
}
