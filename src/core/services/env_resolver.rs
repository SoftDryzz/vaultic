use std::collections::{HashMap, HashSet};

use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::environment::Environment;
use crate::core::models::secret_file::{Line, SecretFile};

/// Resolves environment inheritance (base -> dev/staging/prod).
///
/// Given a config with environment definitions and a set of parsed
/// env files, resolves the full inheritance chain and merges layers
/// from base to leaf. Overlay entries always take precedence.
pub struct EnvResolver;

impl EnvResolver {
    /// Resolve the full inheritance chain for the given environment.
    ///
    /// Walks the `inherits` chain in `config`, collects layers from
    /// root to leaf, and merges them in order (later layers override).
    ///
    /// # Errors
    ///
    /// - `EnvironmentNotFound` if the environment or any parent is not
    ///   defined in the config.
    /// - `CircularInheritance` if the chain contains a cycle.
    pub fn resolve(
        &self,
        name: &str,
        config: &AppConfig,
        files: &HashMap<String, SecretFile>,
    ) -> Result<Environment> {
        let chain = self.build_chain(name, config)?;

        let mut merged = SecretFile {
            lines: Vec::new(),
            source_path: None,
        };

        for layer_name in &chain {
            if let Some(layer_file) = files.get(layer_name.as_str()) {
                merged = Self::merge(&merged, layer_file);
            }
        }

        Ok(Environment {
            name: name.to_string(),
            resolved: merged,
            layers: chain,
        })
    }

    /// Build the ordered inheritance chain from root to the target env.
    ///
    /// For `dev` with `inherits = "base"`, returns `["base", "dev"]`.
    /// For `staging` with `inherits = "shared"` and `shared` with
    /// `inherits = "base"`, returns `["base", "shared", "staging"]`.
    pub fn build_chain(&self, name: &str, config: &AppConfig) -> Result<Vec<String>> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = name.to_string();

        // Walk upward collecting ancestors
        loop {
            if visited.contains(&current) {
                chain.push(current.clone());
                let cycle: Vec<String> = chain.into_iter().rev().collect();
                return Err(VaulticError::CircularInheritance {
                    chain: cycle.join(" -> "),
                });
            }

            let entry = config.environments.get(&current).ok_or_else(|| {
                VaulticError::EnvironmentNotFound {
                    name: current.clone(),
                }
            })?;

            visited.insert(current.clone());
            chain.push(current.clone());

            match &entry.inherits {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        }

        // Reverse so root is first, leaf is last
        chain.reverse();
        Ok(chain)
    }

    /// Merge two secret files: base + overlay.
    ///
    /// 1. Start with all entries from base.
    /// 2. For each entry in overlay:
    ///    - If the key exists in base, replace the value.
    ///    - If it's a new key, append it.
    /// 3. Comments and blanks from overlay are appended after
    ///    base entries to preserve documentation.
    fn merge(base: &SecretFile, overlay: &SecretFile) -> SecretFile {
        let mut lines = base.lines.clone();

        // Build a lookup of existing keys to their index in lines
        let mut key_index: HashMap<String, usize> = HashMap::new();
        for (i, line) in lines.iter().enumerate() {
            if let Line::Entry(entry) = line {
                key_index.insert(entry.key.clone(), i);
            }
        }

        for line in &overlay.lines {
            match line {
                Line::Entry(entry) => {
                    if let Some(&idx) = key_index.get(&entry.key) {
                        // Override existing key
                        lines[idx] = Line::Entry(entry.clone());
                    } else {
                        // New key from overlay
                        key_index.insert(entry.key.clone(), lines.len());
                        lines.push(Line::Entry(entry.clone()));
                    }
                }
                Line::Comment(_) | Line::Blank => {
                    // Overlay comments/blanks are appended
                    lines.push(line.clone());
                }
            }
        }

        SecretFile {
            lines,
            source_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::{AuditSection, EnvEntry, VaulticSection};
    use crate::core::models::secret_file::SecretEntry;

    /// Helper: build a SecretFile from key-value pairs.
    fn make_file(pairs: &[(&str, &str)]) -> SecretFile {
        SecretFile {
            lines: pairs
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    Line::Entry(SecretEntry {
                        key: k.to_string(),
                        value: v.to_string(),
                        comment: None,
                        line_number: i + 1,
                    })
                })
                .collect(),
            source_path: None,
        }
    }

    /// Helper: build a minimal AppConfig with given environments.
    fn make_config(envs: &[(&str, Option<&str>, Option<&str>)]) -> AppConfig {
        let mut environments = HashMap::new();
        for (name, file, inherits) in envs {
            environments.insert(
                name.to_string(),
                EnvEntry {
                    file: file.map(|f| f.to_string()),
                    inherits: inherits.map(|i| i.to_string()),
                },
            );
        }
        AppConfig {
            vaultic: VaulticSection {
                version: "0.1.0".to_string(),
                default_cipher: "age".to_string(),
                default_env: "dev".to_string(),
            },
            environments,
            audit: Some(AuditSection {
                enabled: false,
                log_file: "audit.log".to_string(),
            }),
        }
    }

    #[test]
    fn merge_overlay_overrides_base() {
        let base = make_file(&[("DB", "localhost"), ("PORT", "5432")]);
        let overlay = make_file(&[("DB", "rds.aws.com")]);

        let result = EnvResolver::merge(&base, &overlay);

        assert_eq!(result.get("DB"), Some("rds.aws.com"));
        assert_eq!(result.get("PORT"), Some("5432"));
    }

    #[test]
    fn merge_overlay_adds_new_keys() {
        let base = make_file(&[("DB", "localhost")]);
        let overlay = make_file(&[("REDIS", "redis:6379")]);

        let result = EnvResolver::merge(&base, &overlay);

        assert_eq!(result.get("DB"), Some("localhost"));
        assert_eq!(result.get("REDIS"), Some("redis:6379"));
    }

    #[test]
    fn merge_empty_base() {
        let base = make_file(&[]);
        let overlay = make_file(&[("KEY", "val")]);

        let result = EnvResolver::merge(&base, &overlay);

        assert_eq!(result.keys(), vec!["KEY"]);
    }

    #[test]
    fn merge_empty_overlay() {
        let base = make_file(&[("KEY", "val")]);
        let overlay = make_file(&[]);

        let result = EnvResolver::merge(&base, &overlay);

        assert_eq!(result.keys(), vec!["KEY"]);
    }

    #[test]
    fn resolve_single_level_inheritance() {
        let resolver = EnvResolver;
        let config = make_config(&[
            ("base", Some("base.env"), None),
            ("dev", Some("dev.env"), Some("base")),
        ]);
        let mut files = HashMap::new();
        files.insert(
            "base".to_string(),
            make_file(&[("DB", "localhost"), ("PORT", "5432")]),
        );
        files.insert(
            "dev".to_string(),
            make_file(&[("DB", "dev-db"), ("DEBUG", "true")]),
        );

        let env = resolver.resolve("dev", &config, &files).unwrap();

        assert_eq!(env.name, "dev");
        assert_eq!(env.layers, vec!["base", "dev"]);
        assert_eq!(env.resolved.get("DB"), Some("dev-db"));
        assert_eq!(env.resolved.get("PORT"), Some("5432"));
        assert_eq!(env.resolved.get("DEBUG"), Some("true"));
    }

    #[test]
    fn resolve_multi_level_inheritance() {
        let resolver = EnvResolver;
        let config = make_config(&[
            ("base", Some("base.env"), None),
            ("shared", Some("shared.env"), Some("base")),
            ("dev", Some("dev.env"), Some("shared")),
        ]);
        let mut files = HashMap::new();
        files.insert(
            "base".to_string(),
            make_file(&[("DB", "localhost"), ("PORT", "5432")]),
        );
        files.insert(
            "shared".to_string(),
            make_file(&[("DB", "shared-db"), ("CACHE", "redis")]),
        );
        files.insert("dev".to_string(), make_file(&[("DEBUG", "true")]));

        let env = resolver.resolve("dev", &config, &files).unwrap();

        assert_eq!(env.layers, vec!["base", "shared", "dev"]);
        assert_eq!(env.resolved.get("DB"), Some("shared-db"));
        assert_eq!(env.resolved.get("PORT"), Some("5432"));
        assert_eq!(env.resolved.get("CACHE"), Some("redis"));
        assert_eq!(env.resolved.get("DEBUG"), Some("true"));
    }

    #[test]
    fn resolve_no_inheritance() {
        let resolver = EnvResolver;
        let config = make_config(&[("base", Some("base.env"), None)]);
        let mut files = HashMap::new();
        files.insert("base".to_string(), make_file(&[("KEY", "val")]));

        let env = resolver.resolve("base", &config, &files).unwrap();

        assert_eq!(env.layers, vec!["base"]);
        assert_eq!(env.resolved.get("KEY"), Some("val"));
    }

    #[test]
    fn resolve_circular_inheritance_detected() {
        let resolver = EnvResolver;
        let config = make_config(&[
            ("a", Some("a.env"), Some("b")),
            ("b", Some("b.env"), Some("a")),
        ]);
        let files = HashMap::new();

        let result = resolver.resolve("a", &config, &files);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular inheritance"));
    }

    #[test]
    fn resolve_missing_environment_fails() {
        let resolver = EnvResolver;
        let config = make_config(&[("base", Some("base.env"), None)]);
        let files = HashMap::new();

        let result = resolver.resolve("nonexistent", &config, &files);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn resolve_missing_parent_fails() {
        let resolver = EnvResolver;
        let config = make_config(&[("dev", Some("dev.env"), Some("missing_base"))]);
        let files = HashMap::new();

        let result = resolver.resolve("dev", &config, &files);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing_base"));
    }

    #[test]
    fn resolve_missing_file_uses_empty() {
        let resolver = EnvResolver;
        let config = make_config(&[
            ("base", Some("base.env"), None),
            ("dev", Some("dev.env"), Some("base")),
        ]);
        // Only base has a file, dev file is missing
        let mut files = HashMap::new();
        files.insert("base".to_string(), make_file(&[("DB", "localhost")]));

        let env = resolver.resolve("dev", &config, &files).unwrap();

        // Should still work with just base values
        assert_eq!(env.resolved.get("DB"), Some("localhost"));
    }

    #[test]
    fn build_chain_ordering() {
        let resolver = EnvResolver;
        let config = make_config(&[
            ("base", Some("base.env"), None),
            ("shared", Some("shared.env"), Some("base")),
            ("dev", Some("dev.env"), Some("shared")),
        ]);

        let chain = resolver.build_chain("dev", &config).unwrap();

        assert_eq!(chain, vec!["base", "shared", "dev"]);
    }

    #[test]
    fn merge_preserves_base_comments() {
        let mut base = make_file(&[("DB", "localhost")]);
        base.lines
            .insert(0, Line::Comment("# Database config".to_string()));

        let overlay = make_file(&[("DB", "rds.aws.com")]);

        let result = EnvResolver::merge(&base, &overlay);

        assert!(matches!(result.lines[0], Line::Comment(_)));
        assert_eq!(result.get("DB"), Some("rds.aws.com"));
    }
}
