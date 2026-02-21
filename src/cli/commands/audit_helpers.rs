use std::path::Path;
use std::process::Command;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::adapters::audit::json_audit_logger::JsonAuditLogger;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::models::audit_entry::{AuditAction, AuditEntry};
use crate::core::traits::audit::AuditLogger;

/// Read the git user name and email from the local/global config.
/// Returns `("unknown", None)` if git is not available.
pub fn git_author() -> (String, Option<String>) {
    let name = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let email = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let val = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if val.is_empty() { None } else { Some(val) }
            } else {
                None
            }
        });

    (name, email)
}

/// Compute the SHA-256 hash of a file, returning the hex string.
/// Returns `None` if the file cannot be read.
pub fn compute_file_hash(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let hash = Sha256::digest(&data);
    Some(format!("{hash:x}"))
}

/// Record an audit event. Warns on failure instead of propagating
/// the error, since audit should not block the main operation.
pub fn log_audit(action: AuditAction, files: Vec<String>, detail: Option<String>) {
    log_audit_with_hash(action, files, detail, None);
}

/// Record an audit event with an optional state hash.
pub fn log_audit_with_hash(
    action: AuditAction,
    files: Vec<String>,
    detail: Option<String>,
    state_hash: Option<String>,
) {
    let vaultic_dir = crate::cli::context::vaultic_dir();

    let config = AppConfig::load(vaultic_dir).ok();

    let audit_section = config.as_ref().and_then(|c| c.audit.as_ref());

    if !JsonAuditLogger::is_enabled(audit_section) {
        return;
    }

    let logger = JsonAuditLogger::from_config(vaultic_dir, audit_section);
    let (author, email) = git_author();

    let entry = AuditEntry {
        timestamp: Utc::now(),
        author,
        email,
        action,
        files,
        detail,
        state_hash,
    };

    if let Err(e) = logger.log_event(&entry) {
        output::warning(&format!("Could not write audit log: {e}"));
    }
}

/// Record an audit event right after `vaultic init`, before config
/// exists. Uses default values for the logger path.
pub fn log_audit_init() {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    let logger = JsonAuditLogger::new(vaultic_dir, "audit.log");
    let (author, email) = git_author();

    let entry = AuditEntry {
        timestamp: Utc::now(),
        author,
        email,
        action: AuditAction::Init,
        files: vec![],
        detail: Some("project initialized".to_string()),
        state_hash: None,
    };

    if let Err(e) = logger.log_event(&entry) {
        output::warning(&format!("Could not write audit log: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_file_hash_returns_hex_string() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello world").unwrap();

        let hash = compute_file_hash(&file).unwrap();
        // SHA-256 of "hello world" is well-known
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn compute_file_hash_nonexistent_returns_none() {
        let result = compute_file_hash(Path::new("/nonexistent/file.txt"));
        assert!(result.is_none());
    }
}
