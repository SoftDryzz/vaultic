use crate::core::errors::Result;
use crate::core::models::secret_file::SecretFile;

/// Result of checking a local env file against a template.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub missing: Vec<String>,
    pub extra: Vec<String>,
    pub empty_values: Vec<String>,
}

/// Validates that a local secrets file matches the template.
pub struct CheckService;

impl CheckService {
    /// Compare a local file against a template and report discrepancies.
    pub fn check(&self, _local: &SecretFile, _template: &SecretFile) -> Result<CheckResult> {
        todo!()
    }
}
