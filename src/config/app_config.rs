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
    pub fn load(vaultic_dir: &Path) -> Result<Self> {
        let config_path = vaultic_dir.join("config.toml");
        if !config_path.exists() {
            return Err(VaulticError::InvalidConfig {
                detail: "config.toml not found. Run 'vaultic init' first.".into(),
            });
        }
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| VaulticError::InvalidConfig {
            detail: format!("Failed to parse config.toml: {e}"),
        })
    }

    /// Get the file name for a given environment, defaulting to `{name}.env`.
    pub fn env_file_name(&self, name: &str) -> String {
        self.environments
            .get(name)
            .and_then(|e| e.file.clone())
            .unwrap_or_else(|| format!("{name}.env"))
    }
}

/// The `[vaultic]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct VaulticSection {
    pub version: String,
    pub default_cipher: String,
    pub default_env: String,
}

/// An environment entry in `[environments]`.
#[derive(Debug, Clone, Deserialize)]
pub struct EnvEntry {
    pub file: Option<String>,
    pub inherits: Option<String>,
}

/// The `[audit]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditSection {
    pub enabled: bool,
    pub log_file: String,
}
