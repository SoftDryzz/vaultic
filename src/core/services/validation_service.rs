use std::collections::HashMap;

use regex::Regex;

use crate::config::app_config::ValidationConfig;
use crate::core::errors::{Result, VaulticError};

/// Result of validating a single key.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct KeyValidation {
    pub key: String,
    pub passed: bool,
    /// All failure reasons for this key (can be multiple).
    pub failures: Vec<String>,
}

/// Result of running all validation rules.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ValidationReport {
    pub results: Vec<KeyValidation>,
}

#[allow(dead_code)]
impl ValidationReport {
    /// Count how many keys failed validation.
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    /// Returns `true` if all keys passed (or no rules were checked).
    pub fn is_ok(&self) -> bool {
        self.failed_count() == 0
    }
}

/// Service that validates a map of key→value pairs against a `ValidationConfig`.
pub struct ValidationService;

#[allow(dead_code)]
impl ValidationService {
    /// Validate a map of key→value against validation rules.
    ///
    /// Returns `Err` only for invalid regex patterns in rules (config error).
    /// Missing non-required keys are silently skipped.
    /// All failures for a key are collected before moving to the next.
    pub fn validate(
        values: &HashMap<String, String>,
        rules: &ValidationConfig,
    ) -> Result<ValidationReport> {
        let mut results = Vec::new();

        for (key, rule) in rules {
            let value_opt = values.get(key);

            // Handle required check — if absent or empty, record and skip further checks
            if rule.required {
                let absent_or_empty = match value_opt {
                    None => true,
                    Some(s) => s.is_empty(),
                };
                if absent_or_empty {
                    results.push(KeyValidation {
                        key: key.clone(),
                        passed: false,
                        failures: vec!["required but missing or empty".to_string()],
                    });
                    continue;
                }
            } else if value_opt.is_none() {
                // Non-required key is absent — silently skip
                continue;
            }

            // At this point the key is present (and non-empty if required)
            let val = match value_opt {
                Some(v) => v.as_str(),
                None => continue,
            };

            let mut failures: Vec<String> = Vec::new();

            // --- Type checks ---
            if let Some(type_str) = &rule.value_type {
                match type_str.as_str() {
                    "url" => {
                        if !val.contains("://") {
                            failures.push("expected a valid URL (must contain '://')".to_string());
                        }
                    }
                    "integer" => match val.parse::<i64>() {
                        Ok(n) => {
                            if let Some(min) = rule.min
                                && n < min
                            {
                                failures.push(format!("value {n} is below minimum {min}"));
                            }
                            if let Some(max) = rule.max
                                && n > max
                            {
                                failures.push(format!("value {n} is above maximum {max}"));
                            }
                        }
                        Err(_) => {
                            failures.push(format!("expected integer, got '{val}'"));
                        }
                    },
                    "boolean" => {
                        let lower = val.to_lowercase();
                        if !matches!(lower.as_str(), "true" | "false" | "1" | "0" | "yes" | "no") {
                            failures.push(format!(
                                "expected boolean (true/false/1/0/yes/no), got '{val}'"
                            ));
                        }
                    }
                    _ => {
                        // "string" and unknown types: no type-specific check
                    }
                }
            }

            // --- Length checks ---
            let len = val.len();
            if let Some(min_length) = rule.min_length
                && len < min_length
            {
                failures.push(format!("too short ({len} chars, minimum {min_length})"));
            }
            if let Some(max_length) = rule.max_length
                && len > max_length
            {
                failures.push(format!("too long ({len} chars, maximum {max_length})"));
            }

            // --- Pattern check ---
            if let Some(pattern) = &rule.pattern {
                let re = Regex::new(pattern).map_err(|e| VaulticError::InvalidPattern {
                    key: key.clone(),
                    pattern: pattern.clone(),
                    reason: e.to_string(),
                })?;
                if !re.is_match(val) {
                    failures.push(format!("does not match pattern \"{pattern}\""));
                }
            }

            let passed = failures.is_empty();
            results.push(KeyValidation {
                key: key.clone(),
                passed,
                failures,
            });
        }

        Ok(ValidationReport { results })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::ValidationRule;

    fn make_values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn make_rules(pairs: Vec<(&str, ValidationRule)>) -> ValidationConfig {
        pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    fn url_rule() -> ValidationRule {
        ValidationRule {
            value_type: Some("url".to_string()),
            ..Default::default()
        }
    }

    fn integer_rule(min: Option<i64>, max: Option<i64>) -> ValidationRule {
        ValidationRule {
            value_type: Some("integer".to_string()),
            min,
            max,
            ..Default::default()
        }
    }

    fn boolean_rule() -> ValidationRule {
        ValidationRule {
            value_type: Some("boolean".to_string()),
            ..Default::default()
        }
    }

    fn length_rule(min_length: Option<usize>, max_length: Option<usize>) -> ValidationRule {
        ValidationRule {
            min_length,
            max_length,
            ..Default::default()
        }
    }

    fn pattern_rule(pat: &str) -> ValidationRule {
        ValidationRule {
            pattern: Some(pat.to_string()),
            ..Default::default()
        }
    }

    fn required_rule() -> ValidationRule {
        ValidationRule {
            required: true,
            ..Default::default()
        }
    }

    // ----- URL tests -----

    #[test]
    fn valid_url_passes() {
        let values = make_values(&[("API_URL", "https://example.com")]);
        let rules = make_rules(vec![("API_URL", url_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
        assert!(report.results[0].passed);
    }

    #[test]
    fn invalid_url_fails() {
        let values = make_values(&[("API_URL", "not-a-url")]);
        let rules = make_rules(vec![("API_URL", url_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failures[0].contains("URL"));
    }

    // ----- Integer tests -----

    #[test]
    fn valid_integer_passes() {
        let values = make_values(&[("PORT", "8080")]);
        let rules = make_rules(vec![("PORT", integer_rule(None, None))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn integer_below_min_fails() {
        let values = make_values(&[("PORT", "0")]);
        let rules = make_rules(vec![("PORT", integer_rule(Some(1), None))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("below minimum"));
    }

    #[test]
    fn integer_above_max_fails() {
        let values = make_values(&[("PORT", "99999")]);
        let rules = make_rules(vec![("PORT", integer_rule(None, Some(65535)))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("above maximum"));
    }

    #[test]
    fn non_integer_fails() {
        let values = make_values(&[("PORT", "abc")]);
        let rules = make_rules(vec![("PORT", integer_rule(None, None))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("expected integer"));
    }

    // ----- Boolean tests -----

    #[test]
    fn valid_boolean_true_passes() {
        let values = make_values(&[("DEBUG", "true")]);
        let rules = make_rules(vec![("DEBUG", boolean_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn valid_boolean_false_passes() {
        let values = make_values(&[("DEBUG", "false")]);
        let rules = make_rules(vec![("DEBUG", boolean_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn valid_boolean_1_passes() {
        let values = make_values(&[("DEBUG", "1")]);
        let rules = make_rules(vec![("DEBUG", boolean_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn valid_boolean_0_passes() {
        let values = make_values(&[("DEBUG", "0")]);
        let rules = make_rules(vec![("DEBUG", boolean_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn invalid_boolean_fails() {
        let values = make_values(&[("DEBUG", "maybe")]);
        let rules = make_rules(vec![("DEBUG", boolean_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("expected boolean"));
    }

    // ----- Length tests -----

    #[test]
    fn string_below_min_length_fails() {
        let values = make_values(&[("TOKEN", "ab")]);
        let rules = make_rules(vec![("TOKEN", length_rule(Some(5), None))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("too short"));
    }

    #[test]
    fn string_above_max_length_fails() {
        let values = make_values(&[("TOKEN", "abcdefghij")]);
        let rules = make_rules(vec![("TOKEN", length_rule(None, Some(5)))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("too long"));
    }

    // ----- Pattern tests -----

    #[test]
    fn pattern_match_passes() {
        let values = make_values(&[("KEY", "abc123")]);
        let rules = make_rules(vec![("KEY", pattern_rule(r"^[a-z]+\d+$"))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn pattern_no_match_fails() {
        let values = make_values(&[("KEY", "ABC123")]);
        let rules = make_rules(vec![("KEY", pattern_rule(r"^[a-z]+\d+$"))]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("does not match pattern"));
    }

    #[test]
    fn invalid_regex_returns_error() {
        let values = make_values(&[("KEY", "value")]);
        let rules = make_rules(vec![("KEY", pattern_rule(r"[invalid("))]);
        let result = ValidationService::validate(&values, &rules);
        assert!(result.is_err());
        match result.unwrap_err() {
            VaulticError::InvalidPattern { key, .. } => assert_eq!(key, "KEY"),
            other => panic!("Expected InvalidPattern, got {other:?}"),
        }
    }

    // ----- Required tests -----

    #[test]
    fn required_missing_key_fails() {
        let values = make_values(&[]);
        let rules = make_rules(vec![("SECRET", required_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("required but missing or empty"));
    }

    #[test]
    fn required_empty_value_fails() {
        let values = make_values(&[("SECRET", "")]);
        let rules = make_rules(vec![("SECRET", required_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        assert!(report.results[0].failures[0].contains("required but missing or empty"));
    }

    #[test]
    fn non_required_absent_key_skipped() {
        let values = make_values(&[]);
        let rules = make_rules(vec![("OPTIONAL_KEY", url_rule())]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        // No entry for the absent key
        assert!(report.results.is_empty());
        assert!(report.is_ok());
    }

    // ----- Combined tests -----

    #[test]
    fn type_and_pattern_both_checked_value_passes_pattern_but_fails_min_length() {
        // Value "ab" passes pattern r"\w+" but fails min_length=5
        let values = make_values(&[("KEY", "ab")]);
        let rules = make_rules(vec![(
            "KEY",
            ValidationRule {
                min_length: Some(5),
                pattern: Some(r"\w+".to_string()),
                ..Default::default()
            },
        )]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        // Only 1 failure: min_length
        assert_eq!(report.results[0].failures.len(), 1);
        assert!(report.results[0].failures[0].contains("too short"));
    }

    #[test]
    fn multiple_failures_on_same_key() {
        // Value "ab" fails min_length=5 AND pattern r"^\d+$"
        let values = make_values(&[("KEY", "ab")]);
        let rules = make_rules(vec![(
            "KEY",
            ValidationRule {
                min_length: Some(5),
                pattern: Some(r"^\d+$".to_string()),
                ..Default::default()
            },
        )]);
        let report = ValidationService::validate(&values, &rules).unwrap();
        assert!(!report.is_ok());
        // 2 failures: min_length + pattern
        assert_eq!(report.results[0].failures.len(), 2);
    }
}
