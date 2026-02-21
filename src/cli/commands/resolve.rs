use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::commands::crypto_helpers;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::env_resolver::EnvResolver;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic resolve --env <name>` command.
///
/// Resolves the full inheritance chain for the given environment,
/// decrypting each layer in memory, merging from base to leaf,
/// and writing the result to `.env`.
pub fn execute(env: Option<&str>, cipher: &str) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;
    let env_name = env.unwrap_or(&config.vaultic.default_env);

    output::header(&format!("Resolving environment: {env_name}"));

    let resolver = EnvResolver;
    let parser = DotenvParser;

    // Build the chain first so we know what to decrypt
    let chain = resolver.build_chain(env_name, &config)?;

    output::success(&format!("Inheritance chain: {}", chain.join(" -> ")));

    // Decrypt and parse each layer
    let files = crypto_helpers::load_env_files(&chain, vaultic_dir, cipher, &parser, true)?;

    // Resolve the full inheritance
    let environment = resolver.resolve(env_name, &config, &files)?;

    // Serialize and write to .env
    let content = parser.serialize(&environment.resolved)?;
    let var_count = environment.resolved.keys().len();

    std::fs::write(".env", &content)?;

    output::success(&format!(
        "Resolved {var_count} variables from {} layer(s)",
        environment.layers.len()
    ));
    output::success("Written to .env");
    println!("\n  Run 'vaultic check' to verify against the template.");

    // Audit
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::Resolve,
        vec![format!("{env_name}")],
        Some(format!(
            "{var_count} variables from {} layer(s)",
            environment.layers.len()
        )),
    );

    Ok(())
}
