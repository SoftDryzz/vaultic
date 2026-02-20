use std::collections::HashMap;
use std::path::Path;

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::secret_file::SecretFile;
use crate::core::services::encryption_service::EncryptionService;
use crate::core::services::env_resolver::EnvResolver;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic resolve --env <name>` command.
///
/// Resolves the full inheritance chain for the given environment,
/// decrypting each layer in memory, merging from base to leaf,
/// and writing the result to `.env`.
pub fn execute(env: Option<&str>, cipher: &str) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
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
    let files = load_env_files(&chain, vaultic_dir, cipher, &parser)?;

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

    Ok(())
}

/// Load and decrypt env files for each layer in the chain.
///
/// For each environment name, tries to decrypt the corresponding
/// `.env.enc` file from `.vaultic/`. If the encrypted file doesn't
/// exist, the layer is skipped (it may have no overrides).
fn load_env_files(
    chain: &[String],
    vaultic_dir: &Path,
    cipher: &str,
    parser: &DotenvParser,
) -> Result<HashMap<String, SecretFile>> {
    let mut files = HashMap::new();

    for name in chain {
        // The encrypted file follows the pattern: {name}.env.enc
        let enc_path = vaultic_dir.join(format!("{name}.env.enc"));

        if !enc_path.exists() {
            output::warning(&format!(
                "No encrypted file for '{name}' ({}) — skipping",
                enc_path.display()
            ));
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

/// Decrypt a single encrypted file in memory using the configured cipher.
fn decrypt_in_memory(enc_path: &Path, vaultic_dir: &Path, cipher: &str) -> Result<Vec<u8>> {
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            if !identity_path.exists() {
                return Err(VaulticError::EncryptionFailed {
                    reason: format!(
                        "No private key found at {}\n\n  Run 'vaultic keys setup' to generate a key.",
                        identity_path.display()
                    ),
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
