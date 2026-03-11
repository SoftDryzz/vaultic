use std::path::PathBuf;

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::TemplateAction;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditAction;
use crate::core::services::encryption_service::EncryptionService;
use crate::core::services::template_sync_service::TemplateSyncService;
use crate::core::traits::parser::ConfigParser;

/// Execute `vaultic template` subcommands.
pub fn execute(action: &TemplateAction) -> Result<()> {
    match action {
        TemplateAction::Sync {
            output: output_path,
        } => sync(output_path.as_deref()),
    }
}

/// Implement `vaultic template sync`.
///
/// Decrypts all environments in memory, collects all keys (union),
/// strips values, and writes the result to `.env.template` (or a custom path).
fn sync(output_path: Option<&str>) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;

    // Resolve the identity path — only age is supported for in-memory decryption
    let identity_path = AgeBackend::default_identity_path()?;
    if !identity_path.exists() {
        return Err(VaulticError::EncryptionFailed {
            reason: format!(
                "No private key found at {}\n\n  Solutions:\n    \
                 → New here? Run 'vaultic keys setup' to generate a key\n    \
                 → Have a key? Use --key <path> to specify the location\n    \
                 → Lost your key? Ask an admin to re-add you as a recipient",
                identity_path.display()
            ),
        });
    }

    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));
    let backend = AgeBackend::new(identity_path);
    let service = EncryptionService {
        cipher: backend,
        key_store,
    };
    let parser = DotenvParser;

    let sp = output::spinner("Decrypting environments for template sync...");

    let mut secret_files = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut processed: Vec<String> = Vec::new();

    let mut env_names: Vec<_> = config.environments.keys().cloned().collect();
    env_names.sort();

    for env_name in &env_names {
        let file_name = config.env_file_name(env_name);
        let enc_path = vaultic_dir.join(format!("{file_name}.enc"));

        if !enc_path.exists() {
            output::detail(&format!("Skipping {env_name}: {file_name}.enc not found"));
            skipped.push(env_name.clone());
            continue;
        }

        let plaintext_bytes = service.decrypt_to_bytes(&enc_path)?;
        let content = String::from_utf8_lossy(&plaintext_bytes);
        let secret_file = parser.parse(&content)?;

        output::detail(&format!(
            "Decrypted {env_name}: {} keys",
            secret_file.keys().len()
        ));

        processed.push(env_name.clone());
        secret_files.push(secret_file);
    }

    // Drop spinner before printing results
    if let Some(pb) = sp {
        pb.finish_and_clear();
    }

    if secret_files.is_empty() {
        return Err(VaulticError::InvalidConfig {
            detail: "No encrypted environments found. Run 'vaultic encrypt' first to create encrypted files.".into(),
        });
    }

    output::success(&format!("Decrypted {} environment(s)", processed.len()));

    if !skipped.is_empty() {
        output::warning(&format!(
            "Skipped {} environment(s) (no .enc file): {}",
            skipped.len(),
            skipped.join(", ")
        ));
    }

    // Merge all secret files into a template
    let sync_service = TemplateSyncService;
    let template = sync_service.merge_to_template(&secret_files);
    let key_count = template.keys().len();

    // Serialize the template
    let serialized = parser.serialize(&template)?;

    // Write to output path
    let dest = PathBuf::from(output_path.unwrap_or(".env.template"));
    std::fs::write(&dest, &serialized)?;

    output::success(&format!("Written {} keys to {}", key_count, dest.display()));
    println!("\n  All values stripped — safe to commit.");
    println!("  Run 'vaultic check' to verify your local .env is in sync.");

    // Audit
    let detail = format!(
        "{key_count} keys written from environments: {}",
        processed.join(", ")
    );
    super::audit_helpers::log_audit(
        AuditAction::TemplateSync,
        vec![dest.display().to_string()],
        Some(detail),
    );

    Ok(())
}
