use serde::Deserialize;
use std::collections::HashMap;

/// Top-level Vaultic configuration read from `.vaultic/config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub vaultic: VaulticSection,
    pub environments: HashMap<String, EnvEntry>,
    pub audit: Option<AuditSection>,
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
