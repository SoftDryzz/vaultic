use std::path::Path;

use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::check_service::CheckService;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic check` command.
///
/// Compares the local `.env` against `.env.template` and reports
/// missing, extra, and empty-value variables.
pub fn execute() -> Result<()> {
    let env_path = Path::new(".env");
    let template_path = Path::new(".env.template");

    if !env_path.exists() {
        return Err(VaulticError::FileNotFound {
            path: env_path.to_path_buf(),
        });
    }

    if !template_path.exists() {
        return Err(VaulticError::FileNotFound {
            path: template_path.to_path_buf(),
        });
    }

    let parser = DotenvParser;
    let env_content = std::fs::read_to_string(env_path)?;
    let template_content = std::fs::read_to_string(template_path)?;

    let env_file = parser.parse(&env_content)?;
    let template_file = parser.parse(&template_content)?;

    let svc = CheckService;
    let result = svc.check(&env_file, &template_file)?;

    let total_template = template_file.keys().len();
    let present = total_template - result.missing.len();

    output::header("🔍 vaultic check");

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
        output::success(&format!("{present}/{total_template} variables present"));
    }

    Ok(())
}
