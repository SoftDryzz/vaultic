use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::core::models::audit_entry::{AuditAction, AuditEntry};

/// Result for a single environment rotation check.
#[derive(Debug, Clone)]
pub struct SecretAgeResult {
    /// Environment name (derived from the encrypted file name).
    pub key: String,
    /// When the environment was last encrypted.
    pub last_rotated: Option<DateTime<Utc>>,
    /// Days since last encryption (None if never recorded).
    pub days_since_rotation: Option<i64>,
    /// Whether this exceeds the policy threshold.
    pub exceeds_policy: bool,
}

/// Service that checks how recently each environment was encrypted,
/// compared against a rotation policy (maximum days between rotations).
pub struct SecretAgeService;

impl SecretAgeService {
    /// Given audit log entries and a rotation policy (max days),
    /// return age results for each environment found in Encrypt entries.
    ///
    /// Strategy: scan Encrypt entries, group by env name (derived from
    /// `files[0]`), find the most recent encrypt per env, compute age.
    pub fn check_rotation(
        entries: &[AuditEntry],
        policy_days: u32,
        now: DateTime<Utc>,
    ) -> Vec<SecretAgeResult> {
        // Collect the most recent Encrypt timestamp per env file
        let mut latest: HashMap<String, DateTime<Utc>> = HashMap::new();

        for entry in entries {
            if entry.action != AuditAction::Encrypt {
                continue;
            }
            for file in &entry.files {
                let env_name = Self::env_name_from_file(file);
                latest
                    .entry(env_name)
                    .and_modify(|ts| {
                        if entry.timestamp > *ts {
                            *ts = entry.timestamp;
                        }
                    })
                    .or_insert(entry.timestamp);
            }
        }

        let mut results: Vec<SecretAgeResult> = latest
            .into_iter()
            .map(|(key, ts)| {
                let days = (now - ts).num_days();
                SecretAgeResult {
                    key,
                    last_rotated: Some(ts),
                    days_since_rotation: Some(days),
                    exceeds_policy: days > i64::from(policy_days),
                }
            })
            .collect();

        results.sort_by(|a, b| a.key.cmp(&b.key));
        results
    }

    /// Extract a human-readable env name from a file path like `dev.env.enc`.
    fn env_name_from_file(file: &str) -> String {
        file.trim_end_matches(".enc")
            .trim_end_matches(".env")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::audit_entry::{AuditAction, AuditEntry};

    fn encrypt_entry(days_ago: i64) -> AuditEntry {
        AuditEntry {
            timestamp: Utc::now() - chrono::Duration::days(days_ago),
            author: "test".to_string(),
            email: None,
            action: AuditAction::Encrypt,
            files: vec!["dev.env.enc".to_string()],
            detail: Some("3 variables encrypted".to_string()),
            state_hash: None,
        }
    }

    fn encrypt_entry_for(env_file: &str, days_ago: i64) -> AuditEntry {
        AuditEntry {
            timestamp: Utc::now() - chrono::Duration::days(days_ago),
            author: "test".to_string(),
            email: None,
            action: AuditAction::Encrypt,
            files: vec![env_file.to_string()],
            detail: None,
            state_hash: None,
        }
    }

    #[test]
    fn empty_log_returns_empty() {
        let results = SecretAgeService::check_rotation(&[], 90, Utc::now());
        assert!(results.is_empty());
    }

    #[test]
    fn within_policy_does_not_exceed() {
        let entry = encrypt_entry(30);
        let results = SecretAgeService::check_rotation(&[entry], 90, Utc::now());
        assert_eq!(results.len(), 1);
        assert!(!results[0].exceeds_policy);
        assert_eq!(results[0].key, "dev");
    }

    #[test]
    fn exceeds_policy_flagged() {
        let entry = encrypt_entry(120);
        let results = SecretAgeService::check_rotation(&[entry], 90, Utc::now());
        assert_eq!(results.len(), 1);
        assert!(results[0].exceeds_policy);
    }

    #[test]
    fn most_recent_entry_used() {
        let old = encrypt_entry(100);
        let recent = encrypt_entry(10);
        let results = SecretAgeService::check_rotation(&[old, recent], 90, Utc::now());
        assert_eq!(results.len(), 1);
        assert!(!results[0].exceeds_policy);
    }

    #[test]
    fn multiple_envs_tracked_independently() {
        let dev_recent = encrypt_entry_for("dev.env.enc", 10);
        let prod_old = encrypt_entry_for("prod.env.enc", 120);
        let results = SecretAgeService::check_rotation(&[dev_recent, prod_old], 90, Utc::now());
        assert_eq!(results.len(), 2);
        let dev = results.iter().find(|r| r.key == "dev").unwrap();
        let prod = results.iter().find(|r| r.key == "prod").unwrap();
        assert!(!dev.exceeds_policy);
        assert!(prod.exceeds_policy);
    }

    #[test]
    fn non_encrypt_entries_ignored() {
        let decrypt_entry = AuditEntry {
            timestamp: Utc::now() - chrono::Duration::days(5),
            author: "test".to_string(),
            email: None,
            action: AuditAction::Decrypt,
            files: vec!["dev.env.enc".to_string()],
            detail: None,
            state_hash: None,
        };
        let results = SecretAgeService::check_rotation(&[decrypt_entry], 90, Utc::now());
        assert!(results.is_empty());
    }

    #[test]
    fn env_name_extraction() {
        assert_eq!(SecretAgeService::env_name_from_file("dev.env.enc"), "dev");
        assert_eq!(SecretAgeService::env_name_from_file("prod.env.enc"), "prod");
        assert_eq!(
            SecretAgeService::env_name_from_file("staging.env.enc"),
            "staging"
        );
    }
}
