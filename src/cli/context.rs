use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::core::errors::{Result, VaulticError};

static VAULTIC_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the global vaultic directory path.
/// If `custom` is provided, uses that path; otherwise defaults to `.vaultic`.
pub fn init(custom: Option<&str>) {
    let dir = custom
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".vaultic"));
    let _ = VAULTIC_DIR.set(dir);
}

/// Get the current vaultic directory path.
pub fn vaultic_dir() -> &'static Path {
    VAULTIC_DIR
        .get()
        .map(|p| p.as_path())
        .unwrap_or(Path::new(".vaultic"))
}

/// Validate that an environment name is safe for path construction.
///
/// Prevents path traversal attacks by restricting names to `[a-zA-Z0-9_-]`.
/// For example, `--env ../../../etc` would construct `.vaultic/../../../etc.env.enc`
/// and escape the project directory.
pub fn validate_env_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(VaulticError::InvalidConfig {
            detail: "Environment name cannot be empty.\n\n  \
                     Use a name like 'dev', 'staging', or 'prod'."
                .into(),
        });
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(VaulticError::InvalidConfig {
            detail: format!(
                "Invalid environment name: '{name}'\n\n  \
                 Environment names can only contain letters, digits, hyphens, and underscores.\n  \
                 Valid examples: 'dev', 'staging', 'prod-us', 'test_01'"
            ),
        });
    }

    Ok(())
}

/// Validate that a filename is safe and does not contain path separators.
///
/// Prevents a compromised `config.toml` from writing files outside `.vaultic/`.
pub fn validate_simple_filename(name: &str, context: &str) -> Result<()> {
    if name.is_empty() {
        return Err(VaulticError::InvalidConfig {
            detail: format!("{context} cannot be empty."),
        });
    }

    if name.contains('/') || name.contains('\\') || name.contains("..") || name.starts_with('.') {
        return Err(VaulticError::InvalidConfig {
            detail: format!(
                "Unsafe {context}: '{name}'\n\n  \
                 The value must be a simple filename without path separators.\n  \
                 Valid examples: 'audit.log', 'vaultic-audit.log'"
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_env_names() {
        assert!(validate_env_name("dev").is_ok());
        assert!(validate_env_name("staging").is_ok());
        assert!(validate_env_name("prod").is_ok());
        assert!(validate_env_name("prod-us").is_ok());
        assert!(validate_env_name("test_01").is_ok());
        assert!(validate_env_name("A").is_ok());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_env_name("../../../etc").is_err());
        assert!(validate_env_name("..").is_err());
        assert!(validate_env_name("foo/bar").is_err());
        assert!(validate_env_name("foo\\bar").is_err());
    }

    #[test]
    fn rejects_empty_env_name() {
        assert!(validate_env_name("").is_err());
    }

    #[test]
    fn rejects_special_characters() {
        assert!(validate_env_name("dev;rm -rf").is_err());
        assert!(validate_env_name("prod env").is_err());
        assert!(validate_env_name("dev.staging").is_err());
    }

    #[test]
    fn valid_simple_filenames() {
        assert!(validate_simple_filename("audit.log", "log file").is_ok());
        assert!(validate_simple_filename("vaultic-audit.log", "log file").is_ok());
        assert!(validate_simple_filename("log", "log file").is_ok());
    }

    #[test]
    fn rejects_path_in_filename() {
        assert!(validate_simple_filename("../secret.log", "log file").is_err());
        assert!(validate_simple_filename("/etc/passwd", "log file").is_err());
        assert!(validate_simple_filename("foo\\bar.log", "log file").is_err());
        assert!(validate_simple_filename("..\\..\\etc", "log file").is_err());
    }

    #[test]
    fn rejects_hidden_files() {
        assert!(validate_simple_filename(".hidden", "log file").is_err());
    }

    #[test]
    fn rejects_empty_filename() {
        assert!(validate_simple_filename("", "log file").is_err());
    }
}
