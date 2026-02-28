use std::path::PathBuf;

/// All domain errors for Vaultic.
///
/// Each variant provides enough context to diagnose the issue
/// without needing a debugger.
#[derive(Debug, thiserror::Error)]
pub enum VaulticError {
    #[error(
        "File not found: {path}\n\n  \
         Check that the path is correct and the file exists.\n  \
         Run 'vaultic status' to see available environments and files."
    )]
    FileNotFound { path: PathBuf },

    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error(
        "Decryption failed: no matching key found\n\n  \
         Your private key is not in the recipients list for this file.\n\n  \
         Solutions:\n    \
         → Ask a project admin to add your public key: vaultic keys add <your-key>\n    \
         → Then re-encrypt: vaultic encrypt --all\n    \
         → Check your recipient status: vaultic status"
    )]
    DecryptionNoKey,

    #[error(
        "Parse error in {file}: {detail}\n\n  \
         Expected format: KEY=value (one per line).\n  \
         Comments (#) and blank lines are allowed."
    )]
    ParseError { file: PathBuf, detail: String },

    #[error(
        "Environment '{name}' not found\n\n  \
         Available environments: {available}\n  \
         Check .vaultic/config.toml for environment definitions."
    )]
    EnvironmentNotFound { name: String, available: String },

    #[error(
        "Circular inheritance detected: {chain}\n\n  \
         Two or more environments inherit from each other, creating a loop.\n\n  \
         Fix: edit .vaultic/config.toml and ensure inheritance forms a tree:\n    \
         → Valid:   base → dev, base → staging, base → prod\n    \
         → Invalid: dev → staging → dev (cycle)"
    )]
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

    #[error(
        "Update check failed: {reason}\n\n  \
         This is not critical — your current version continues to work.\n  \
         Try again later or check https://github.com/SoftDryzz/vaultic/releases"
    )]
    UpdateCheckFailed { reason: String },

    #[error(
        "Update verification failed: {reason}\n\n  \
         The downloaded binary could not be verified and was NOT installed.\n  \
         Your current installation is unchanged.\n\n  \
         Solutions:\n    \
         → Try again: vaultic update\n    \
         → Manual download: https://github.com/SoftDryzz/vaultic/releases/latest\n    \
         → Report issue: https://github.com/SoftDryzz/vaultic/issues"
    )]
    UpdateVerificationFailed { reason: String },

    #[error(
        "Update failed: {reason}\n\n  \
         The binary replacement failed. Your current installation may be intact.\n\n  \
         Solutions:\n    \
         → Try again: vaultic update\n    \
         → Manual install: cargo install vaultic --force"
    )]
    UpdateFailed { reason: String },

    #[error(
        "Unsupported platform for auto-update: {platform}\n\n  \
         Pre-built binaries are not available for your platform.\n\n  \
         Solutions:\n    \
         → Install from source: cargo install vaultic\n    \
         → Build manually: cargo build --release"
    )]
    UnsupportedPlatform { platform: String },

    #[error(
        "No template file found\n\n  \
         Vaultic searched for:\n    \
         {searched}\n\n  \
         Solutions:\n    \
         → Create a template: cp .env .env.template (then remove secret values)\n    \
         → Specify in .vaultic/config.toml:\n      \
           [vaultic]\n      \
           template = \"path/to/your/template\""
    )]
    TemplateNotFound { searched: String },

    #[error(
        "This project uses format version {project_version}, but your Vaultic \
         only supports up to version {supported_version}.\n\n  \
         Solutions:\n    \
         → Update Vaultic: vaultic update\n    \
         → Or install latest: cargo install vaultic --force"
    )]
    FormatVersionTooNew {
        project_version: u32,
        supported_version: u32,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, VaulticError>;
