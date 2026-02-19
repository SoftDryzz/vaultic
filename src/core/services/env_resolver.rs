use crate::core::errors::Result;
use crate::core::models::environment::Environment;
use crate::core::models::secret_file::SecretFile;

/// Resolves environment inheritance (base → dev/staging/prod).
pub struct EnvResolver;

impl EnvResolver {
    /// Merge a base file with an environment overlay.
    ///
    /// Entries in `overlay` take precedence over `base`.
    pub fn resolve(
        &self,
        _name: &str,
        _base: &SecretFile,
        _overlay: &SecretFile,
    ) -> Result<Environment> {
        todo!()
    }
}
