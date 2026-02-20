use std::collections::HashMap;
use std::path::Path;

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::secret_file::SecretFile;
use crate::core::services::encryption_service::EncryptionService;
use crate::core::traits::parser::ConfigParser;

/// Load and decrypt env files for each layer in the chain.
///
/// For each environment name, tries to decrypt the corresponding
/// `.env.enc` file from `.vaultic/`. If the encrypted file doesn't
/// exist, the layer is skipped (it may have no overrides).
///
/// When `warn_missing` is true, prints a warning for missing files.
pub fn load_env_files(
    chain: &[String],
    vaultic_dir: &Path,
    cipher: &str,
    parser: &DotenvParser,
    warn_missing: bool,
) -> Result<HashMap<String, SecretFile>> {
    let mut files = HashMap::new();

    for name in chain {
        let enc_path = vaultic_dir.join(format!("{name}.env.enc"));

        if !enc_path.exists() {
            if warn_missing {
                crate::cli::output::warning(&format!(
                    "No encrypted file for '{name}' ({}) — skipping",
                    enc_path.display()
                ));
            }
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
pub fn decrypt_in_memory(enc_path: &Path, vaultic_dir: &Path, cipher: &str) -> Result<Vec<u8>> {
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
