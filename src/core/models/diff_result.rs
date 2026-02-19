/// Classification of a single variable difference between two files.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    Added,
    Removed,
    Modified {
        old_value: String,
        new_value: String,
    },
}

/// One entry in a diff comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffEntry {
    pub key: String,
    pub kind: DiffKind,
}

/// Result of comparing two secret files or environments.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub left_name: String,
    pub right_name: String,
    pub entries: Vec<DiffEntry>,
}

impl DiffResult {
    /// Returns true if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
