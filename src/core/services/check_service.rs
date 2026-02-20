use std::collections::BTreeSet;

use crate::core::errors::Result;
use crate::core::models::secret_file::SecretFile;

/// Result of checking a local env file against a template.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// Variables in the template but missing from the local file.
    pub missing: Vec<String>,
    /// Variables in the local file but not in the template.
    pub extra: Vec<String>,
    /// Variables present in the local file but with empty values.
    pub empty_values: Vec<String>,
}

impl CheckResult {
    /// Returns true if the local file is fully in sync with the template.
    pub fn is_ok(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty() && self.empty_values.is_empty()
    }

    /// Total number of issues found.
    pub fn issue_count(&self) -> usize {
        self.missing.len() + self.extra.len() + self.empty_values.len()
    }
}

/// Validates that a local secrets file matches the template.
pub struct CheckService;

impl CheckService {
    /// Compare a local file against a template and report discrepancies.
    ///
    /// - **Missing**: keys in `template` that are absent from `local`
    /// - **Extra**: keys in `local` that are absent from `template`
    /// - **Empty values**: keys present in `local` with an empty string value
    ///
    /// All result vectors are sorted alphabetically.
    pub fn check(&self, local: &SecretFile, template: &SecretFile) -> Result<CheckResult> {
        let local_keys: BTreeSet<&str> = local.keys().into_iter().collect();
        let template_keys: BTreeSet<&str> = template.keys().into_iter().collect();

        let missing: Vec<String> = template_keys
            .difference(&local_keys)
            .map(|k| k.to_string())
            .collect();

        let extra: Vec<String> = local_keys
            .difference(&template_keys)
            .map(|k| k.to_string())
            .collect();

        let empty_values: Vec<String> = local
            .entries()
            .filter(|e| e.value.is_empty())
            .map(|e| e.key.clone())
            .collect();

        Ok(CheckResult {
            missing,
            extra,
            empty_values,
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
    fn all_present_no_issues() {
        let svc = CheckService;
        let local = make_file(&[("DB", "localhost"), ("PORT", "5432")]);
        let template = make_file(&[("DB", ""), ("PORT", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert!(result.missing.is_empty());
        assert!(result.extra.is_empty());
        assert!(result.is_ok());
    }

    #[test]
    fn detects_missing_variables() {
        let svc = CheckService;
        let local = make_file(&[("DB", "localhost")]);
        let template = make_file(&[("DB", ""), ("API_KEY", ""), ("SECRET", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert_eq!(result.missing, vec!["API_KEY", "SECRET"]);
        assert!(result.extra.is_empty());
    }

    #[test]
    fn detects_extra_variables() {
        let svc = CheckService;
        let local = make_file(&[("DB", "localhost"), ("OLD_VAR", "legacy")]);
        let template = make_file(&[("DB", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert!(result.missing.is_empty());
        assert_eq!(result.extra, vec!["OLD_VAR"]);
    }

    #[test]
    fn detects_empty_values() {
        let svc = CheckService;
        let local = make_file(&[("DB", "localhost"), ("API_KEY", ""), ("SECRET", "")]);
        let template = make_file(&[("DB", ""), ("API_KEY", ""), ("SECRET", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert!(result.missing.is_empty());
        assert_eq!(result.empty_values, vec!["API_KEY", "SECRET"]);
    }

    #[test]
    fn mixed_issues() {
        let svc = CheckService;
        let local = make_file(&[("DB", "localhost"), ("OLD", "x"), ("EMPTY", "")]);
        let template = make_file(&[("DB", ""), ("EMPTY", ""), ("NEW_VAR", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert_eq!(result.missing, vec!["NEW_VAR"]);
        assert_eq!(result.extra, vec!["OLD"]);
        assert_eq!(result.empty_values, vec!["EMPTY"]);
        assert_eq!(result.issue_count(), 3);
    }

    #[test]
    fn empty_local_reports_all_missing() {
        let svc = CheckService;
        let local = make_file(&[]);
        let template = make_file(&[("A", ""), ("B", "")]);
        let result = svc.check(&local, &template).unwrap();

        assert_eq!(result.missing, vec!["A", "B"]);
    }

    #[test]
    fn empty_template_reports_all_extra() {
        let svc = CheckService;
        let local = make_file(&[("A", "1"), ("B", "2")]);
        let template = make_file(&[]);
        let result = svc.check(&local, &template).unwrap();

        assert_eq!(result.extra, vec!["A", "B"]);
    }
}
