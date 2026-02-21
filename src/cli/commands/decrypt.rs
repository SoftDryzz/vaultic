use std::path::{Path, PathBuf};

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::cipher::gpg_backend::GpgBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::encryption_service::EncryptionService;
use crate::core::traits::cipher::CipherBackend;

/// Execute the `vaultic decrypt` command.
///
/// Decrypts an encrypted file from `.vaultic/` and writes
/// the plaintext to the working directory.
/// When `key_path` is provided, uses that file as the private key
/// instead of the default location.
pub fn execute(
    file: Option<&str>,
    env: Option<&str>,
    cipher: &str,
    key_path: Option<&str>,
) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let env_name = env.unwrap_or("dev");
    let source = match file {
        Some(f) => PathBuf::from(f),
        None => vaultic_dir.join(format!("{env_name}.env.enc")),
    };

    if !source.exists() {
        return Err(VaulticError::FileNotFound {
            path: source.clone(),
        });
    }

    let dest = PathBuf::from(".env");
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    match cipher {
        "age" => {
            let identity_path = match key_path {
                Some(p) => {
                    let path = PathBuf::from(p);
                    if !path.exists() {
                        return Err(VaulticError::FileNotFound { path });
                    }
                    path
                }
                None => {
                    let path = AgeBackend::default_identity_path()?;
                    if !path.exists() {
                        return Err(VaulticError::EncryptionFailed {
                            reason: format!(
                                "No private key found at {}\n\n  Solutions:\n    \
                                 → New here? Run 'vaultic keys setup' to generate a key\n    \
                                 → Have a key? Use --key <path> to specify the location\n    \
                                 → Lost your key? Ask an admin to re-add you as a recipient",
                                path.display()
                            ),
                        });
                    }
                    path
                }
            };
            let backend = AgeBackend::new(identity_path);
            decrypt_with(backend, key_store, &source, &dest, env_name)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            if !backend.is_available() {
                return Err(VaulticError::EncryptionFailed {
                    reason: "GPG is not installed or not found in PATH".into(),
                });
            }
            decrypt_with(backend, key_store, &source, &dest, env_name)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Decrypt with a given backend.
fn decrypt_with<C: CipherBackend>(
    cipher: C,
    key_store: FileKeyStore,
    source: &Path,
    dest: &Path,
    env_name: &str,
) -> Result<()> {
    let cipher_name = cipher.name().to_string();

    let service = EncryptionService { cipher, key_store };

    output::header(&format!("Decrypting {env_name} with {cipher_name}"));
    output::detail(&format!("Source: {}", source.display()));
    output::detail(&format!("Destination: {}", dest.display()));

    service.decrypt_file(source, dest)?;

    // Count variables in decrypted file
    let content = std::fs::read_to_string(dest)?;
    let var_count = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && t.contains('=')
        })
        .count();

    output::success(&format!("Decrypted {}", source.display()));
    output::success(&format!("Generated .env with {var_count} variables"));
    println!("\n  Run 'vaultic check' to verify no variables are missing.");

    // Audit
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::Decrypt,
        vec![format!("{env_name}.env.enc")],
        Some(format!("{var_count} variables decrypted")),
    );

    Ok(())
}
