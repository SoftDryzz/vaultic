use std::collections::BTreeSet;

use crate::core::errors::Result;
use crate::core::models::diff_result::{DiffEntry, DiffKind, DiffResult};
use crate::core::models::secret_file::SecretFile;

/// Compares two secret files and produces a structured diff.
pub struct DiffService;

impl DiffService {
    /// Compare two `SecretFile`s and return their differences.
    ///
    /// - Keys only in `left` are `Removed`
    /// - Keys only in `right` are `Added`
    /// - Keys in both with different values are `Modified`
    /// - Keys in both with the same value are omitted (no diff)
    ///
    /// Results are sorted alphabetically by key.
    pub fn diff(
        &self,
        left: &SecretFile,
        right: &SecretFile,
        left_name: &str,
        right_name: &str,
    ) -> Result<DiffResult> {
        let left_keys: BTreeSet<&str> = left.keys().into_iter().collect();
        let right_keys: BTreeSet<&str> = right.keys().into_iter().collect();

        let mut entries = Vec::new();

        // All unique keys, sorted via BTreeSet
        let all_keys: BTreeSet<&str> = left_keys.union(&right_keys).copied().collect();

        for key in all_keys {
            let left_val = left.get(key);
            let right_val = right.get(key);

            match (left_val, right_val) {
                (Some(_), None) => {
                    entries.push(DiffEntry {
                        key: key.to_string(),
                        kind: DiffKind::Removed,
                    });
                }
                (None, Some(_)) => {
                    entries.push(DiffEntry {
                        key: key.to_string(),
                        kind: DiffKind::Added,
                    });
                }
                (Some(old), Some(new)) if old != new => {
                    entries.push(DiffEntry {
                        key: key.to_string(),
                        kind: DiffKind::Modified {
                            old_value: old.to_string(),
                            new_value: new.to_string(),
                        },
                    });
                }
                _ => {} // Same value — no diff
            }
        }

        Ok(DiffResult {
            left_name: left_name.to_string(),
            right_name: right_name.to_string(),
            entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::secret_file::{Line, SecretEntry};

    /// Helper to build a SecretFile from key-value pairs.
    fn make_file(pairs: &[(&str, &str)]) -> SecretFile {
        SecretFile {
            lines: pairs
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    Line::Entry(SecretEntry {
                        key: k.to_string(),
                        value: v.to_string(),
                        comment: None,
                        line_number: i + 1,
                    })
                })
                .collect(),
            source_path: None,
        }
    }

    #[test]
    fn identical_files_produce_empty_diff() {
        let svc = DiffService;
        let a = make_file(&[("DB", "localhost"), ("PORT", "5432")]);
        let b = make_file(&[("DB", "localhost"), ("PORT", "5432")]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn detects_added_keys() {
        let svc = DiffService;
        let a = make_file(&[("DB", "localhost")]);
        let b = make_file(&[("DB", "localhost"), ("REDIS", "redis:6379")]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].key, "REDIS");
        assert_eq!(result.entries[0].kind, DiffKind::Added);
    }

    #[test]
    fn detects_removed_keys() {
        let svc = DiffService;
        let a = make_file(&[("DB", "localhost"), ("OLD_KEY", "gone")]);
        let b = make_file(&[("DB", "localhost")]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].key, "OLD_KEY");
        assert_eq!(result.entries[0].kind, DiffKind::Removed);
    }

    #[test]
    fn detects_modified_values() {
        let svc = DiffService;
        let a = make_file(&[("DB", "localhost")]);
        let b = make_file(&[("DB", "rds.aws.com")]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].key, "DB");
        assert_eq!(
            result.entries[0].kind,
            DiffKind::Modified {
                old_value: "localhost".to_string(),
                new_value: "rds.aws.com".to_string(),
            }
        );
    }

    #[test]
    fn mixed_changes() {
        let svc = DiffService;
        let a = make_file(&[("A", "1"), ("B", "old"), ("C", "3")]);
        let b = make_file(&[("B", "new"), ("C", "3"), ("D", "4")]);
        let result = svc.diff(&a, &b, "left", "right").unwrap();

        assert_eq!(result.entries.len(), 3);
        // Sorted alphabetically: A (removed), B (modified), D (added)
        assert_eq!(result.entries[0].key, "A");
        assert_eq!(result.entries[0].kind, DiffKind::Removed);
        assert_eq!(result.entries[1].key, "B");
        assert!(matches!(result.entries[1].kind, DiffKind::Modified { .. }));
        assert_eq!(result.entries[2].key, "D");
        assert_eq!(result.entries[2].kind, DiffKind::Added);
    }

    #[test]
    fn preserves_file_names() {
        let svc = DiffService;
        let a = make_file(&[("X", "1")]);
        let b = make_file(&[("X", "2")]);
        let result = svc.diff(&a, &b, "dev.env", "prod.env").unwrap();

        assert_eq!(result.left_name, "dev.env");
        assert_eq!(result.right_name, "prod.env");
    }

    #[test]
    fn keys_are_case_sensitive() {
        let svc = DiffService;
        let a = make_file(&[("key", "lower")]);
        let b = make_file(&[("KEY", "upper")]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        // "key" and "KEY" are different variables
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].key, "KEY");
        assert_eq!(result.entries[0].kind, DiffKind::Added);
        assert_eq!(result.entries[1].key, "key");
        assert_eq!(result.entries[1].kind, DiffKind::Removed);
    }

    #[test]
    fn empty_files_produce_empty_diff() {
        let svc = DiffService;
        let a = make_file(&[]);
        let b = make_file(&[]);
        let result = svc.diff(&a, &b, "a", "b").unwrap();

        assert!(result.is_empty());
    }
}
