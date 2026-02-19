use super::secret_file::SecretFile;

/// Represents an environment (dev, staging, prod) with its
/// resolved configuration after applying inheritance.
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub name: String,
    pub resolved: SecretFile,
    pub layers: Vec<String>,
}
