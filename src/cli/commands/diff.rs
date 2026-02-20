use std::collections::HashMap;
use std::path::Path;

use colored::Colorize;

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::diff_result::{DiffKind, DiffResult};
use crate::core::models::secret_file::SecretFile;
use crate::core::services::diff_service::DiffService;
use crate::core::services::encryption_service::EncryptionService;
use crate::core::services::env_resolver::EnvResolver;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic diff` command.
///
/// Two modes:
/// - File mode:  `vaultic diff file1 file2`
/// - Env mode:   `vaultic diff --env dev --env prod`
pub fn execute(
    file1: Option<&str>,
    file2: Option<&str>,
    envs: &[String],
    cipher: &str,
) -> Result<()> {
    if envs.len() >= 2 {
        execute_env_diff(&envs[0], &envs[1], cipher)
    } else {
        execute_file_diff(file1, file2)
    }
}

/// Compare two resolved environments.
fn execute_env_diff(left_env: &str, right_env: &str, cipher: &str) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;
    let resolver = EnvResolver;
    let parser = DotenvParser;

    output::header(&format!(
        "Comparing environments: {left_env} vs {right_env}"
    ));

    // Resolve left environment
    let left_chain = resolver.build_chain(left_env, &config)?;
    let left_files = load_env_files(&left_chain, vaultic_dir, cipher, &parser)?;
    let left = resolver.resolve(left_env, &config, &left_files)?;

    // Resolve right environment
    let right_chain = resolver.build_chain(right_env, &config)?;
    let right_files = load_env_files(&right_chain, vaultic_dir, cipher, &parser)?;
    let right = resolver.resolve(right_env, &config, &right_files)?;

    let svc = DiffService;
    let result = svc.diff(&left.resolved, &right.resolved, left_env, right_env)?;

    if result.is_empty() {
        output::success("No differences found between environments");
        return Ok(());
    }

    print_diff_table(&result);
    print_diff_summary(&result);

    Ok(())
}

/// Compare two plain files.
fn execute_file_diff(file1: Option<&str>, file2: Option<&str>) -> Result<()> {
    let left_path = file1.unwrap_or(".env");
    let right_path = file2.ok_or_else(|| VaulticError::InvalidConfig {
        detail: "diff requires two files. Usage: vaultic diff <file1> <file2>".to_string(),
    })?;

    let left = Path::new(left_path);
    let right = Path::new(right_path);

    if !left.exists() {
        return Err(VaulticError::FileNotFound {
            path: left.to_path_buf(),
        });
    }
    if !right.exists() {
        return Err(VaulticError::FileNotFound {
            path: right.to_path_buf(),
        });
    }

    let parser = DotenvParser;
    let left_content = std::fs::read_to_string(left)?;
    let right_content = std::fs::read_to_string(right)?;

    let left_file = parser.parse(&left_content)?;
    let right_file = parser.parse(&right_content)?;

    let svc = DiffService;
    let result = svc.diff(&left_file, &right_file, left_path, right_path)?;

    output::header("vaultic diff");

    if result.is_empty() {
        output::success("No differences found");
        return Ok(());
    }

    print_diff_table(&result);
    print_diff_summary(&result);

    Ok(())
}

/// Load and decrypt env files for each layer in the chain.
fn load_env_files(
    chain: &[String],
    vaultic_dir: &Path,
    cipher: &str,
    parser: &DotenvParser,
) -> Result<HashMap<String, SecretFile>> {
    let mut files = HashMap::new();

    for name in chain {
        let enc_path = vaultic_dir.join(format!("{name}.env.enc"));

        if !enc_path.exists() {
            continue;
        }

        let plaintext_bytes = decrypt_in_memory(&enc_path, vaultic_dir, cipher)?;
        let plaintext =
            String::from_utf8(plaintext_bytes).map_err(|_| VaulticError::ParseError {
                file: enc_path.clone(),
                detail: "Decrypted content is not valid UTF-8".into(),
            })?;

        let secret_file = parser.parse(&plaintext)?;
        files.insert(name.clone(), secret_file);
    }

    Ok(files)
}

/// Decrypt a single encrypted file in memory.
fn decrypt_in_memory(enc_path: &Path, vaultic_dir: &Path, cipher: &str) -> Result<Vec<u8>> {
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            if !identity_path.exists() {
                return Err(VaulticError::EncryptionFailed {
                    reason: format!("No private key found at {}", identity_path.display()),
                });
            }
            let backend = AgeBackend::new(identity_path);
            let service = EncryptionService {
                cipher: backend,
                key_store,
            };
            service.decrypt_to_bytes(enc_path)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Print the diff results as a formatted table.
fn print_diff_table(result: &DiffResult) {
    let key_width = result
        .entries
        .iter()
        .map(|e| e.key.len())
        .max()
        .unwrap_or(8)
        .max(8);

    let header = format!(
        "  {:<width$}   {:<12}   {}",
        "Variable",
        &result.left_name,
        &result.right_name,
        width = key_width
    );
    println!("{}", header.bold());
    println!("  {}", "─".repeat(header.len()));

    for entry in &result.entries {
        match &entry.kind {
            DiffKind::Added => {
                println!(
                    "  {:<width$}   {:<12}   {}",
                    entry.key.green(),
                    "—".dimmed(),
                    "(added)".green(),
                    width = key_width
                );
            }
            DiffKind::Removed => {
                println!(
                    "  {:<width$}   {:<12}   {}",
                    entry.key.red(),
                    "(removed)".red(),
                    "—".dimmed(),
                    width = key_width
                );
            }
            DiffKind::Modified {
                old_value,
                new_value,
            } => {
                let old_display = truncate(old_value, 12);
                let new_display = truncate(new_value, 12);
                println!(
                    "  {:<width$}   {:<12}   {}",
                    entry.key.yellow(),
                    old_display,
                    new_display.yellow(),
                    width = key_width
                );
            }
        }
    }
}

/// Print a summary line below the table.
fn print_diff_summary(result: &DiffResult) {
    let added = result
        .entries
        .iter()
        .filter(|e| matches!(e.kind, DiffKind::Added))
        .count();
    let removed = result
        .entries
        .iter()
        .filter(|e| matches!(e.kind, DiffKind::Removed))
        .count();
    let modified = result
        .entries
        .iter()
        .filter(|e| matches!(e.kind, DiffKind::Modified { .. }))
        .count();

    let mut parts = Vec::new();
    if added > 0 {
        parts.push(format!("{added} added"));
    }
    if removed > 0 {
        parts.push(format!("{removed} removed"));
    }
    if modified > 0 {
        parts.push(format!("{modified} modified"));
    }

    println!();
    output::success(&parts.join(", "));
}

/// Truncate a string to `max_len` characters, appending "..." if needed.
/// Uses char boundaries to avoid panic on multibyte UTF-8 sequences.
fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let limit = max_len.saturating_sub(3);
        let truncated: String = s.chars().take(limit).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn truncate_unicode_safe() {
        // "contraseña" has 10 chars but 11 bytes (ñ = 2 bytes)
        let result = truncate("contraseña", 8);
        assert_eq!(result, "contr...");
        // Should not panic
        let _ = truncate("日本語テスト", 5);
    }

    #[test]
    fn truncate_empty_string() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_max_len_zero() {
        assert_eq!(truncate("hello", 0), "...");
    }
}
