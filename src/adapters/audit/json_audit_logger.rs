use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditEntry;
use crate::core::traits::audit::AuditLogger;

/// Audit logger that appends entries as JSON lines to a file.
///
/// Each line in the log file is a self-contained JSON object representing
/// one `AuditEntry`. This format supports efficient append operations
/// and line-by-line streaming reads.
pub struct JsonAuditLogger {
    log_path: PathBuf,
}

impl JsonAuditLogger {
    /// Create a logger that writes to `{vaultic_dir}/{log_file}`.
    pub fn new(vaultic_dir: &Path, log_file: &str) -> Self {
        Self {
            log_path: vaultic_dir.join(log_file),
        }
    }

    /// Create a logger from an `AppConfig`, falling back to defaults
    /// if the `[audit]` section is missing.
    pub fn from_config(
        vaultic_dir: &Path,
        audit_section: Option<&crate::config::app_config::AuditSection>,
    ) -> Self {
        let log_file = audit_section
            .map(|a| a.log_file.as_str())
            .unwrap_or("audit.log");
        Self::new(vaultic_dir, log_file)
    }

    /// Check whether auditing is enabled in the configuration.
    /// Returns `true` when the section is absent (enabled by default).
    pub fn is_enabled(audit_section: Option<&crate::config::app_config::AuditSection>) -> bool {
        audit_section.map(|a| a.enabled).unwrap_or(true)
    }
}

impl AuditLogger for JsonAuditLogger {
    fn log_event(&self, entry: &AuditEntry) -> Result<()> {
        let line = serde_json::to_string(entry).map_err(|e| VaulticError::AuditError {
            detail: format!("Failed to serialize audit entry: {e}"),
        })?;

        // Ensure the parent directory exists
        if let Some(parent) = self.log_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| VaulticError::AuditError {
                detail: format!("Cannot open audit log at {}: {e}", self.log_path.display()),
            })?;

        writeln!(file, "{line}").map_err(|e| VaulticError::AuditError {
            detail: format!("Failed to write audit entry: {e}"),
        })?;

        Ok(())
    }

    fn query(&self, author: Option<&str>, since: Option<DateTime<Utc>>) -> Result<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.log_path).map_err(|e| VaulticError::AuditError {
            detail: format!("Cannot read audit log: {e}"),
        })?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| VaulticError::AuditError {
                detail: format!("Error reading audit log line {}: {e}", line_num + 1),
            })?;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let entry: AuditEntry =
                serde_json::from_str(trimmed).map_err(|e| VaulticError::AuditError {
                    detail: format!("Malformed audit entry at line {}: {e}", line_num + 1),
                })?;

            // Apply filters
            if let Some(author_filter) = author {
                let author_lower = author_filter.to_lowercase();
                let matches_name = entry.author.to_lowercase().contains(&author_lower);
                let matches_email = entry
                    .email
                    .as_ref()
                    .is_some_and(|e| e.to_lowercase().contains(&author_lower));
                if !matches_name && !matches_email {
                    continue;
                }
            }

            if let Some(since_date) = since
                && entry.timestamp < since_date
            {
                continue;
            }

            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::audit_entry::AuditAction;
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn sample_entry(author: &str, action: AuditAction) -> AuditEntry {
        AuditEntry {
            timestamp: Utc::now(),
            author: author.to_string(),
            email: Some(format!("{author}@test.com")),
            action,
            files: vec!["dev.env".to_string()],
            detail: None,
            state_hash: None,
        }
    }

    #[test]
    fn log_and_query_round_trip() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        let entry = sample_entry("Alice", AuditAction::Encrypt);
        logger.log_event(&entry).unwrap();

        let results = logger.query(None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "Alice");
        assert_eq!(results[0].action, AuditAction::Encrypt);
    }

    #[test]
    fn multiple_entries_appended() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        logger
            .log_event(&sample_entry("Alice", AuditAction::Encrypt))
            .unwrap();
        logger
            .log_event(&sample_entry("Bob", AuditAction::Decrypt))
            .unwrap();
        logger
            .log_event(&sample_entry("Alice", AuditAction::Resolve))
            .unwrap();

        let results = logger.query(None, None).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn filter_by_author() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        logger
            .log_event(&sample_entry("Alice", AuditAction::Encrypt))
            .unwrap();
        logger
            .log_event(&sample_entry("Bob", AuditAction::Decrypt))
            .unwrap();

        let results = logger.query(Some("alice"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "Alice");
    }

    #[test]
    fn filter_by_author_email() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        logger
            .log_event(&sample_entry("Alice", AuditAction::Init))
            .unwrap();

        let results = logger.query(Some("alice@test.com"), None).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn filter_by_since() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        let old = AuditEntry {
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            ..sample_entry("Alice", AuditAction::Init)
        };
        let recent = AuditEntry {
            timestamp: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            ..sample_entry("Bob", AuditAction::Encrypt)
        };

        logger.log_event(&old).unwrap();
        logger.log_event(&recent).unwrap();

        let cutoff = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let results = logger.query(None, Some(cutoff)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "Bob");
    }

    #[test]
    fn query_empty_log_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let logger = JsonAuditLogger::new(tmp.path(), "audit.log");

        let results = logger.query(None, None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn query_nonexistent_file_returns_empty() {
        let logger = JsonAuditLogger::new(Path::new("/nonexistent"), "audit.log");

        let results = logger.query(None, None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn is_enabled_defaults_to_true() {
        assert!(JsonAuditLogger::is_enabled(None));
    }

    #[test]
    fn is_enabled_respects_config() {
        use crate::config::app_config::AuditSection;

        let enabled = AuditSection {
            enabled: true,
            log_file: "audit.log".to_string(),
        };
        let disabled = AuditSection {
            enabled: false,
            log_file: "audit.log".to_string(),
        };

        assert!(JsonAuditLogger::is_enabled(Some(&enabled)));
        assert!(!JsonAuditLogger::is_enabled(Some(&disabled)));
    }
}
