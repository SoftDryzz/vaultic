use crate::core::errors::Result;
use crate::core::models::audit_entry::AuditEntry;

/// Port for recording and querying audit events.
pub trait AuditLogger: Send + Sync {
    /// Append an entry to the audit log.
    fn log_event(&self, entry: &AuditEntry) -> Result<()>;

    /// Query all entries, optionally filtered.
    fn query(
        &self,
        author: Option<&str>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<AuditEntry>>;
}
