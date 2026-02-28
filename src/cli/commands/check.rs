use std::path::Path;

use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::check_service::CheckService;
use crate::core::services::template_resolver::TemplateResolver;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic check` command.
///
/// Compares the local `.env` against the template file and reports
/// missing, extra, and empty-value variables.
///
/// The template is resolved using a priority chain:
/// 1. `template` in config.toml (if configured)
/// 2. Auto-discovery: `.env.template`, `.env.example`, `.env.sample`, `env.template`
pub fn execute() -> Result<()> {
    let env_path = Path::new(".env");

    if !env_path.exists() {
        return Err(VaulticError::FileNotFound {
            path: env_path.to_path_buf(),
        });
    }

    // Load config if available (non-fatal — check works without .vaultic/)
    let project_root = Path::new(".");
    let vaultic_dir = Path::new(".vaultic");
    let config = if vaultic_dir.exists() {
        AppConfig::load(vaultic_dir).ok()
    } else {
        None
    };

    let template_path = TemplateResolver::resolve_global(config.as_ref(), project_root)?;

    let parser = DotenvParser;
    let env_content = std::fs::read_to_string(env_path)?;
    let template_content = std::fs::read_to_string(&template_path)?;

    let env_file = parser.parse(&env_content)?;
    let template_file = parser.parse(&template_content)?;

    let svc = CheckService;
    let result = svc.check(&env_file, &template_file)?;

    let total_template = template_file.keys().len();
    let present = total_template - result.missing.len();

    output::header("🔍 vaultic check");
    output::detail(&format!("Template: {}", template_path.display()));

    if !result.missing.is_empty() {
        output::warning(&format!("Missing variables ({}):", result.missing.len()));
        for key in &result.missing {
            println!("    • {key}");
        }
    }

    if !result.extra.is_empty() {
        output::warning(&format!(
            "Extra variables not in template ({}):",
            result.extra.len()
        ));
        for key in &result.extra {
            println!("    • {key}");
        }
    }

    if !result.empty_values.is_empty() {
        output::warning(&format!(
            "Variables with empty values ({}):",
            result.empty_values.len()
        ));
        for key in &result.empty_values {
            println!("    • {key}");
        }
    }

    if result.is_ok() {
        output::success(&format!(
            "{present}/{total_template} variables present — all good"
        ));
    } else {
        println!();
        output::success(&format!(
            "{present}/{total_template} variables present, {} issue(s) found",
            result.issue_count()
        ));
    }

    // Audit
    let detail = if result.is_ok() {
        format!("{present}/{total_template} present")
    } else {
        format!(
            "{present}/{total_template} present, {} missing",
            result.missing.len()
        )
    };
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::Check,
        vec![".env".to_string()],
        Some(detail),
    );

    Ok(())
}
