use std::path::Path;

use chrono::{NaiveDate, TimeZone, Utc};
use colored::Colorize;

use crate::adapters::audit::json_audit_logger::JsonAuditLogger;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::{AuditAction, AuditEntry};
use crate::core::traits::audit::AuditLogger;

/// Execute the `vaultic log` command.
///
/// Displays the audit log with optional filters for author, date,
/// and entry count.
pub fn execute(author: Option<&str>, since: Option<&str>, last: Option<usize>) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;
    let audit_section = config.audit.as_ref();
    let logger = JsonAuditLogger::from_config(vaultic_dir, audit_section);

    // Parse the --since flag as a date
    let since_dt = since.map(parse_since).transpose()?;

    let entries = logger.query(author, since_dt)?;

    if entries.is_empty() {
        output::header("vaultic log");
        output::warning("No audit entries found");
        if author.is_some() || since.is_some() {
            println!("  Try removing filters to see all entries.");
        }
        return Ok(());
    }

    // Apply --last N (take from the end)
    let display: Vec<&AuditEntry> = match last {
        Some(n) => entries
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        None => entries.iter().collect(),
    };

    output::header(&format!("vaultic log ({} entries)", display.len()));
    println!();

    for entry in &display {
        print_entry(entry);
    }

    Ok(())
}

/// Parse a date string (ISO 8601: `YYYY-MM-DD`) into a UTC DateTime.
fn parse_since(s: &str) -> Result<chrono::DateTime<Utc>> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| VaulticError::InvalidConfig {
            detail: format!(
                "Invalid date format: '{s}'. Expected ISO 8601 (YYYY-MM-DD), e.g. 2026-01-15"
            ),
        })
        .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).expect("midnight is always valid")))
}

/// Print a single audit entry as a formatted row.
fn print_entry(entry: &AuditEntry) {
    let date = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
    let action = format_action(&entry.action);
    let files = if entry.files.is_empty() {
        "—".dimmed().to_string()
    } else {
        entry.files.join(", ")
    };
    let detail = entry.detail.as_deref().unwrap_or("").dimmed().to_string();

    println!(
        "  {} {} {:<10} {} {}",
        date.to_string().dimmed(),
        "│".dimmed(),
        action,
        files,
        detail,
    );
}

/// Format an AuditAction as a colored string.
fn format_action(action: &AuditAction) -> String {
    match action {
        AuditAction::Init => "init".cyan().to_string(),
        AuditAction::Encrypt => "encrypt".green().to_string(),
        AuditAction::Decrypt => "decrypt".blue().to_string(),
        AuditAction::KeyAdd => "key add".green().to_string(),
        AuditAction::KeyRemove => "key rm".red().to_string(),
        AuditAction::Check => "check".yellow().to_string(),
        AuditAction::Diff => "diff".yellow().to_string(),
        AuditAction::Resolve => "resolve".blue().to_string(),
    }
}
