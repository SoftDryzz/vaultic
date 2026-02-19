use crate::core::errors::Result;
use crate::core::models::secret_file::SecretFile;

/// Port for parsing and serializing configuration files.
///
/// v1.0 only ships with `DotenvParser`; the trait enables future
/// support for TOML, YAML, JSON, etc.
pub trait ConfigParser: Send + Sync {
    /// Parse raw file content into a structured `SecretFile`.
    fn parse(&self, content: &str) -> Result<SecretFile>;

    /// Serialize a `SecretFile` back to its file format.
    fn serialize(&self, secrets: &SecretFile) -> Result<String>;

    /// File extensions this parser handles (e.g. `[".env"]`).
    fn supported_extensions(&self) -> &[&str];
}
