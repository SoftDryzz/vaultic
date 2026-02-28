use std::path::{Path, PathBuf};

use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};

/// Priority list of template file names for auto-discovery.
const TEMPLATE_CANDIDATES: &[&str] = &[
    ".env.template",
    ".env.example",
    ".env.sample",
    "env.template",
];

/// Resolves the template file path for a given context.
///
/// Supports global auto-discovery, config-based paths, and
/// per-environment templates with a fallback chain.
pub struct TemplateResolver;

impl TemplateResolver {
    /// Resolve the template path for a global check (no specific environment).
    ///
    /// Resolution order:
    /// 1. Global `template` field in config (if provided and file exists)
    /// 2. Auto-discovery in project root
    pub fn resolve_global(config: Option<&AppConfig>, project_root: &Path) -> Result<PathBuf> {
        // 1. Check config global template
        if let Some(cfg) = config
            && let Some(ref tpl) = cfg.vaultic.template
        {
            let path = project_root.join(tpl);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. Auto-discovery
        Self::auto_discover(project_root)
    }

    /// Resolve the template path for a specific environment.
    ///
    /// Resolution order:
    /// 1. `template` field in environment config (explicit)
    /// 2. `{env}.env.template` convention in `.vaultic/`
    /// 3. Global `template` field in config
    /// 4. Auto-discovery in project root
    #[allow(dead_code)]
    pub fn resolve_for_env(
        env_name: &str,
        config: &AppConfig,
        vaultic_dir: &Path,
        project_root: &Path,
    ) -> Result<PathBuf> {
        // 1. Per-environment template from config
        if let Some(env_entry) = config.environments.get(env_name)
            && let Some(ref tpl) = env_entry.template
        {
            let path = vaultic_dir.join(tpl);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. Convention: {env}.env.template in .vaultic/
        let convention_path = vaultic_dir.join(format!("{env_name}.env.template"));
        if convention_path.exists() {
            return Ok(convention_path);
        }

        // 3. Global template from config
        if let Some(ref tpl) = config.vaultic.template {
            let path = project_root.join(tpl);
            if path.exists() {
                return Ok(path);
            }
        }

        // 4. Auto-discovery fallback
        Self::auto_discover(project_root).map_err(|_| {
            let mut searched = Vec::new();
            if let Some(env_entry) = config.environments.get(env_name)
                && let Some(ref tpl) = env_entry.template
            {
                searched.push(format!(
                    "✗ {} (from config)",
                    vaultic_dir.join(tpl).display()
                ));
            }
            searched.push(format!("✗ {} (convention)", convention_path.display()));
            if let Some(ref tpl) = config.vaultic.template {
                searched.push(format!("✗ {tpl} (global config)"));
            }
            for candidate in TEMPLATE_CANDIDATES {
                searched.push(format!("✗ {candidate} (auto-discovery)"));
            }
            VaulticError::TemplateNotFound {
                searched: searched.join("\n    "),
            }
        })
    }

    /// Auto-discover a template file in the given directory.
    fn auto_discover(base: &Path) -> Result<PathBuf> {
        for candidate in TEMPLATE_CANDIDATES {
            let path = base.join(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        let searched = TEMPLATE_CANDIDATES
            .iter()
            .map(|c| format!("✗ {c}"))
            .collect::<Vec<_>>()
            .join("\n    ");

        Err(VaulticError::TemplateNotFound { searched })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_candidates_have_correct_priority() {
        assert_eq!(TEMPLATE_CANDIDATES[0], ".env.template");
        assert_eq!(TEMPLATE_CANDIDATES[1], ".env.example");
        assert_eq!(TEMPLATE_CANDIDATES[2], ".env.sample");
        assert_eq!(TEMPLATE_CANDIDATES[3], "env.template");
    }

    #[test]
    fn auto_discover_fails_in_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = TemplateResolver::auto_discover(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn auto_discover_finds_env_template() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".env.template"), "KEY=\n").unwrap();
        let result = TemplateResolver::auto_discover(dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dir.path().join(".env.template"));
    }

    #[test]
    fn auto_discover_finds_env_example_as_fallback() {
        let dir = tempfile::tempdir().unwrap();
        // No .env.template, but .env.example exists
        std::fs::write(dir.path().join(".env.example"), "KEY=\n").unwrap();
        let result = TemplateResolver::auto_discover(dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dir.path().join(".env.example"));
    }
}
