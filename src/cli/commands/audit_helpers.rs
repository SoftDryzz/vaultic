use std::path::Path;
use std::process::Command;

use chrono::Utc;

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

/// Record an audit event. Warns on failure instead of propagating
/// the error, since audit should not block the main operation.
pub fn log_audit(action: AuditAction, files: Vec<String>, detail: Option<String>) {
    let vaultic_dir = Path::new(".vaultic");

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
        state_hash: None,
    };

    if let Err(e) = logger.log_event(&entry) {
        output::warning(&format!("Could not write audit log: {e}"));
    }
}

/// Record an audit event right after `vaultic init`, before config
/// exists. Uses default values for the logger path.
pub fn log_audit_init() {
    let vaultic_dir = Path::new(".vaultic");
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
