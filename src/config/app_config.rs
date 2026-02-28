use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::core::errors::{Result, VaulticError};

/// Top-level Vaultic configuration read from `.vaultic/config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub vaultic: VaulticSection,
    pub environments: HashMap<String, EnvEntry>,
    pub audit: Option<AuditSection>,
}

impl AppConfig {
    /// Load the configuration from `.vaultic/config.toml`.
    ///
    /// After parsing, validates environment names and the audit log filename
    /// to prevent path traversal attacks from a compromised config file.
    pub fn load(vaultic_dir: &Path) -> Result<Self> {
        let config_path = vaultic_dir.join("config.toml");
        if !config_path.exists() {
            return Err(VaulticError::InvalidConfig {
                detail: "config.toml not found. Run 'vaultic init' first.".into(),
            });
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content).map_err(|e| VaulticError::InvalidConfig {
            detail: format!("Failed to parse config.toml: {e}"),
        })?;

        // Check format version compatibility
        if config.vaultic.format_version > CURRENT_FORMAT_VERSION {
            return Err(VaulticError::FormatVersionTooNew {
                project_version: config.vaultic.format_version,
                supported_version: CURRENT_FORMAT_VERSION,
            });
        }

        // Validate environment names from config
        for env_name in config.environments.keys() {
            crate::cli::context::validate_env_name(env_name)?;
        }

        // Validate audit log filename
        if let Some(audit) = &config.audit {
            crate::cli::context::validate_simple_filename(&audit.log_file, "audit log file")?;
        }

        Ok(config)
    }

    /// Get the file name for a given environment, defaulting to `{name}.env`.
    pub fn env_file_name(&self, name: &str) -> String {
        self.environments
            .get(name)
            .and_then(|e| e.file.clone())
            .unwrap_or_else(|| format!("{name}.env"))
    }
}

/// Current format version supported by this build of Vaultic.
pub const CURRENT_FORMAT_VERSION: u32 = 1;

/// The `[vaultic]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct VaulticSection {
    pub version: String,
    /// Format version for backward compatibility. Defaults to 1 if missing.
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    pub default_cipher: String,
    pub default_env: String,
    /// Global template file path (optional).
    pub template: Option<String>,
}

fn default_format_version() -> u32 {
    1
}

/// An environment entry in `[environments]`.
#[derive(Debug, Clone, Deserialize)]
pub struct EnvEntry {
    pub file: Option<String>,
    pub inherits: Option<String>,
    /// Per-environment template file (optional).
    pub template: Option<String>,
}

/// The `[audit]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditSection {
    pub enabled: bool,
    pub log_file: String,
}
