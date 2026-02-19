use serde::{Deserialize, Serialize};

/// Actions that get recorded in the audit log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Init,
    Encrypt,
    Decrypt,
    KeyAdd,
    KeyRemove,
    Check,
    Diff,
    Resolve,
}

/// A single entry in the audit log (JSON lines format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub email: Option<String>,
    pub action: AuditAction,
    pub files: Vec<String>,
    pub detail: Option<String>,
    pub state_hash: Option<String>,
}
