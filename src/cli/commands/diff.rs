use std::path::Path;

use colored::Colorize;

use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::diff_result::{DiffKind, DiffResult};
use crate::core::services::diff_service::DiffService;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic diff` command.
///
/// Compares two `.env` files and displays added, removed, and
/// modified variables in a formatted table.
pub fn execute(file1: Option<&str>, file2: Option<&str>, _env: Option<&str>) -> Result<()> {
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

    output::header("🔍 vaultic diff");

    if result.is_empty() {
        output::success("No differences found");
        return Ok(());
    }

    print_diff_table(&result);
    print_diff_summary(&result);

    Ok(())
}

/// Print the diff results as a formatted table.
fn print_diff_table(result: &DiffResult) {
    // Calculate column widths
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
                // Truncate long values for readability
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
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
