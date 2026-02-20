use std::path::Path;

use colored::Colorize;

use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::key_service::KeyService;

/// Execute the `vaultic status` command.
///
/// Displays a full overview of the project state: configuration,
/// keys, encrypted environments, and local file status.
pub fn execute() -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;

    // Project info
    output::header("Vaultic v0.1.0");
    println!("  Cipher: {}", config.vaultic.default_cipher.cyan());
    println!("  Default env: {}", config.vaultic.default_env.cyan());
    println!("  Config: .vaultic/config.toml");

    // Recipients
    print_recipients(vaultic_dir);

    // Encrypted environments
    print_environments(&config, vaultic_dir);

    // Local state
    print_local_state();

    // Audit status
    print_audit_status(&config, vaultic_dir);

    Ok(())
}

/// Print the recipients section.
fn print_recipients(vaultic_dir: &Path) {
    let store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));
    let service = KeyService { store };

    match service.list_keys() {
        Ok(keys) if keys.is_empty() => {
            println!();
            output::warning("No recipients configured");
            println!("  Run 'vaultic keys add <public-key>' to add one.");
        }
        Ok(keys) => {
            println!("\n{}", format!("  Recipients ({})", keys.len()).bold());
            for ki in &keys {
                let display = truncate_key(&ki.public_key, 40);
                println!("  {} {display}", "•".dimmed());
            }
        }
        Err(_) => {
            output::warning("Could not read recipients");
        }
    }
}

/// Print the encrypted environments section.
fn print_environments(config: &AppConfig, vaultic_dir: &Path) {
    println!("\n{}", "  Encrypted environments".bold());

    let mut envs: Vec<_> = config.environments.keys().collect();
    envs.sort();

    for env_name in envs {
        let file_name = config.env_file_name(env_name);
        let enc_path = vaultic_dir.join(format!("{file_name}.enc"));

        if enc_path.exists() {
            let meta = std::fs::metadata(&enc_path).ok();
            let size = meta.as_ref().map(|m| format_bytes(m.len())).unwrap_or_default();
            println!(
                "  {} {:<12} {} {}",
                "✓".green(),
                env_name,
                format!("{file_name}.enc").dimmed(),
                size.dimmed(),
            );
        } else {
            println!(
                "  {} {:<12} {}",
                "✗".red(),
                env_name,
                "(not encrypted)".dimmed(),
            );
        }
    }
}

/// Print local file status (.env, .env.template, .gitignore).
fn print_local_state() {
    println!("\n{}", "  Local state".bold());

    // .env
    let env_path = Path::new(".env");
    if env_path.exists() {
        let content = std::fs::read_to_string(env_path).unwrap_or_default();
        let var_count = count_variables(&content);
        output::success(&format!(".env present ({var_count} variables)"));
    } else {
        output::warning(".env not found");
    }

    // .env.template
    let template_path = Path::new(".env.template");
    if template_path.exists() {
        let content = std::fs::read_to_string(template_path).unwrap_or_default();
        let var_count = count_variables(&content);
        output::success(&format!(".env.template present ({var_count} variables)"));
    } else {
        output::warning(".env.template not found");
    }

    // .gitignore
    let gitignore = Path::new(".gitignore");
    if gitignore.exists() {
        let content = std::fs::read_to_string(gitignore).unwrap_or_default();
        if content.lines().any(|l| l.trim() == ".env") {
            output::success(".env in .gitignore");
        } else {
            output::warning(".env NOT in .gitignore — secrets may be committed!");
        }
    } else {
        output::warning("No .gitignore found");
    }
}

/// Print audit log status.
fn print_audit_status(config: &AppConfig, vaultic_dir: &Path) {
    let audit = config.audit.as_ref();
    let enabled = audit.map(|a| a.enabled).unwrap_or(true);

    if !enabled {
        println!("\n{}", "  Audit: disabled".dimmed());
        return;
    }

    let log_file = audit.map(|a| a.log_file.as_str()).unwrap_or("audit.log");
    let log_path = vaultic_dir.join(log_file);

    if log_path.exists() {
        let content = std::fs::read_to_string(&log_path).unwrap_or_default();
        let entry_count = content.lines().filter(|l| !l.trim().is_empty()).count();
        println!(
            "\n  {} Audit: {} entries in {}",
            "✓".green(),
            entry_count,
            log_file,
        );
    } else {
        println!("\n  {} Audit: no entries yet ({})", "—".dimmed(), log_file);
    }
}

/// Count variable definitions in a dotenv-style string.
fn count_variables(content: &str) -> usize {
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && t.contains('=')
        })
        .count()
}

/// Truncate a key string for display, showing start and end.
fn truncate_key(key: &str, max_len: usize) -> String {
    if key.len() <= max_len {
        key.to_string()
    } else {
        let keep = max_len.saturating_sub(3) / 2;
        format!("{}...{}", &key[..keep], &key[key.len() - keep..])
    }
}

/// Format a byte count as a human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("({bytes} B)")
    } else {
        format!("({:.1} KB)", bytes as f64 / 1024.0)
    }
}
