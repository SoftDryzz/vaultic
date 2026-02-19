use crate::core::errors::Result;
use crate::core::models::diff_result::DiffResult;
use crate::core::models::secret_file::SecretFile;

/// Compares two secret files and produces a structured diff.
pub struct DiffService;

impl DiffService {
    /// Compare two `SecretFile`s and return their differences.
    pub fn diff(&self, _left: &SecretFile, _right: &SecretFile) -> Result<DiffResult> {
        todo!()
    }
}
