use std::path::PathBuf;

/// All domain errors for Vaultic.
///
/// Each variant provides enough context to diagnose the issue
/// without needing a debugger.
#[derive(Debug, thiserror::Error)]
pub enum VaulticError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Decryption failed: no matching key found")]
    DecryptionNoKey,

    #[error("Parse error in {file}: {detail}")]
    ParseError { file: PathBuf, detail: String },

    #[error("Environment '{name}' not found")]
    EnvironmentNotFound { name: String },

    #[error("Circular inheritance detected: {chain}")]
    CircularInheritance { chain: String },

    #[error("Key '{identity}' not found in recipients")]
    KeyNotFound { identity: String },

    #[error("Key '{identity}' already exists in recipients")]
    KeyAlreadyExists { identity: String },

    #[error("Invalid configuration: {detail}")]
    InvalidConfig { detail: String },

    #[error("Audit log error: {detail}")]
    AuditError { detail: String },

    #[error("Git hook error: {detail}")]
    HookError { detail: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, VaulticError>;
