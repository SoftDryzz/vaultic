use std::collections::HashSet;

use crate::core::models::secret_file::{Line, SecretEntry, SecretFile};

/// Merges multiple [`SecretFile`] instances into a single template file.
///
/// The resulting file contains the union of all keys from the input files,
/// with all values set to the empty string `""`. Insertion order is preserved:
/// keys from the first file appear first; subsequent files only contribute
/// keys that have not been seen yet.
pub struct TemplateSyncService;

impl TemplateSyncService {
    /// Merge `files` into a single template `SecretFile`.
    ///
    /// - Keys are deduplicated; the first occurrence determines position.
    /// - All values in the result are set to `""`.
    /// - Comments and blank lines from the source files are **not** copied.
    /// - An empty slice returns an empty `SecretFile`.
    pub fn merge_to_template(&self, files: &[SecretFile]) -> SecretFile {
        let mut seen: HashSet<String> = HashSet::new();
        let mut lines: Vec<Line> = Vec::new();
        let mut line_number: usize = 1;

        for file in files {
            for entry in file.entries() {
                if seen.insert(entry.key.clone()) {
                    lines.push(Line::Entry(SecretEntry {
                        key: entry.key.clone(),
                        value: String::new(),
                        comment: None,
                        line_number,
                    }));
                    line_number += 1;
                }
            }
        }

        SecretFile {
            lines,
            source_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::secret_file::{Line, SecretEntry};

    /// Helper to build a `SecretFile` from key-value pairs.
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
    fn merge_single_file_strips_values() {
        let svc = TemplateSyncService;
        let file = make_file(&[("DB_URL", "postgres://localhost"), ("API_KEY", "secret123")]);
        let result = svc.merge_to_template(&[file]);

        let keys = result.keys();
        assert_eq!(keys, vec!["DB_URL", "API_KEY"]);

        for entry in result.entries() {
            assert_eq!(entry.value, "", "value for {} should be empty", entry.key);
        }
    }

    #[test]
    fn merge_union_all_keys() {
        let svc = TemplateSyncService;
        let dev = make_file(&[("A", "1"), ("B", "2")]);
        let prod = make_file(&[("B", "20"), ("C", "30")]);
        let result = svc.merge_to_template(&[dev, prod]);

        let keys = result.keys();
        assert_eq!(keys, vec!["A", "B", "C"]);
    }

    #[test]
    fn merge_empty_input_returns_empty() {
        let svc = TemplateSyncService;
        let result = svc.merge_to_template(&[]);

        assert!(result.lines.is_empty());
        assert!(result.keys().is_empty());
        assert!(result.source_path.is_none());
    }

    #[test]
    fn merge_preserves_first_key_order() {
        let svc = TemplateSyncService;
        let first = make_file(&[("A", "val_a"), ("B", "val_b")]);
        let second = make_file(&[("C", "val_c"), ("A", "other_a")]);
        let result = svc.merge_to_template(&[first, second]);

        let keys = result.keys();
        // A and B come from the first file, C is appended from the second.
        assert_eq!(keys, vec!["A", "B", "C"]);
    }

    #[test]
    fn merge_duplicate_keys_not_doubled() {
        let svc = TemplateSyncService;
        let file1 = make_file(&[("X", "1"), ("Y", "2")]);
        let file2 = make_file(&[("Y", "3"), ("X", "4"), ("Z", "5")]);
        let result = svc.merge_to_template(&[file1, file2]);

        let keys = result.keys();
        assert_eq!(keys, vec!["X", "Y", "Z"]);
        assert_eq!(keys.len(), 3, "duplicate keys must not appear twice");
    }
}
