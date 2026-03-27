use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::commands::crypto_helpers;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditAction;
use crate::core::services::env_resolver::EnvResolver;


/// Execute `vaultic ci export`.
///
/// Resolves the environment, then prints secrets to stdout in the
/// requested CI format. No files are written to disk.
pub fn execute_export(env: Option<&str>, cipher: &str, format: &str, mask: bool) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    // Validate format
    if !matches!(format, "github" | "gitlab" | "generic") {
        return Err(VaulticError::CiExportFailed {
            format: format.to_string(),
        });
    }

    // --mask only makes sense with github format
    if mask && format != "github" {
        return Err(VaulticError::InvalidConfig {
            detail: "--mask is only supported with --format github".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;
    let env_name = env.unwrap_or(&config.vaultic.default_env);
    let parser = DotenvParser;
    let resolver = EnvResolver;

    // Build inheritance chain and decrypt layers
    let chain = resolver.build_chain(env_name, &config)?;
    let files = crypto_helpers::load_env_files(&chain, vaultic_dir, cipher, &parser, false)?;
    let environment = resolver.resolve(env_name, &config, &files)?;

    // Extract key-value pairs from resolved environment.
    let entries: Vec<(&str, &str)> = environment
        .resolved
        .entries()
        .map(|e| (e.key.as_str(), e.value.as_str()))
        .collect();

    // Format and print to stdout
    for (key, value) in &entries {
        match format {
            "github" => {
                if mask {
                    println!("echo \"::add-mask::{value}\"");
                }
                println!("echo \"{key}={value}\" >> \"$GITHUB_ENV\"");
            }
            "gitlab" => {
                println!("export {key}=\"{value}\"");
            }
            "generic" => {
                println!("{key}={value}");
            }
            _ => unreachable!(),
        }
    }

    // Audit (non-blocking)
    super::audit_helpers::log_audit(
        AuditAction::CiExport,
        vec![env_name.to_string()],
        Some(format!(
            "{} variables exported as {format}",
            entries.len()
        )),
    );

    Ok(())
}
