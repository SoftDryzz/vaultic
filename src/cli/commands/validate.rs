use std::collections::HashMap;
use std::path::Path;

use colored::Colorize;

use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditAction;
use crate::core::services::validation_service::ValidationService;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic validate` command.
///
/// Validates a `.env` file against the format rules defined in
/// `.vaultic/config.toml` under the `[validation]` section.
///
/// Exits with an error (code 1) if any rules fail, making it suitable
/// for use in CI pipelines.
pub fn execute(file: Option<&str>) -> Result<()> {
    let file_path_str = file.unwrap_or(".env");
    let env_path = Path::new(file_path_str);

    if !env_path.exists() {
        return Err(VaulticError::FileNotFound {
            path: env_path.to_path_buf(),
        });
    }

    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;

    let rules = match &config.validation {
        Some(r) if !r.is_empty() => r.clone(),
        _ => {
            output::header("🔍 vaultic validate");
            output::warning("No [validation] rules found in .vaultic/config.toml");
            println!();
            println!("  Add rules like this to .vaultic/config.toml:");
            println!();
            println!("    [validation]");
            println!("    DATABASE_URL = {{ type = \"url\", required = true }}");
            println!("    PORT         = {{ type = \"integer\", min = 1024, max = 65535 }}");
            println!("    API_KEY      = {{ type = \"string\", min_length = 32 }}");
            println!("    DEBUG        = {{ type = \"boolean\" }}");
            println!("    STRIPE_KEY   = {{ pattern = \"^sk_live_.*\" }}");
            return Ok(());
        }
    };

    let rule_count = rules.len();

    // Parse the .env file
    let parser = DotenvParser;
    let content = std::fs::read_to_string(env_path)?;
    let secret_file = parser.parse(&content)?;

    // Build a key → value map for validation
    let values: HashMap<String, String> = secret_file
        .entries()
        .map(|e| (e.key.clone(), e.value.clone()))
        .collect();

    let report = ValidationService::validate(&values, &rules)?;

    output::header("🔍 vaultic validate");
    println!("  File: {file_path_str}");
    println!("  Rules: {rule_count} defined");
    println!();

    // Sort results: failures first (alphabetically), then passes (alphabetically)
    let mut failures: Vec<_> = report.results.iter().filter(|r| !r.passed).collect();
    let mut passes: Vec<_> = report.results.iter().filter(|r| r.passed).collect();
    failures.sort_by(|a, b| a.key.cmp(&b.key));
    passes.sort_by(|a, b| a.key.cmp(&b.key));

    for result in &failures {
        let reason = result.failures.join("; ");
        println!("  {} {} — {}", "✗".red(), result.key.red(), reason);
    }

    for result in &passes {
        output::success(&format!("{} — ok", result.key));
    }

    println!();

    let passed_count = passes.len();
    let failed_count = failures.len();
    println!("  {passed_count} passed, {failed_count} failed");

    // Audit
    let detail = format!("{passed_count} passed, {failed_count} failed");
    super::audit_helpers::log_audit(
        AuditAction::Validate,
        vec![file_path_str.to_string()],
        Some(detail),
    );

    if failed_count > 0 {
        println!();
        println!("  Fix the values in your .env and run 'vaultic validate' again.");
        return Err(VaulticError::ValidationFailed { count: failed_count });
    }

    Ok(())
}
