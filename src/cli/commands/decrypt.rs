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
/// the plaintext to the working directory (or to `output_path` if provided).
/// When `key_path` is provided, uses that file as the private key
/// instead of the default location.
pub fn execute(
    file: Option<&str>,
    env: Option<&str>,
    cipher: &str,
    key_path: Option<&str>,
    output_path: Option<&str>,
    to_stdout: bool,
) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
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

    let dest = match output_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(".env"),
    };
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    match cipher {
        "age" => {
            let backend = match key_path {
                Some(p) => {
                    let path = PathBuf::from(p);
                    if !path.exists() {
                        return Err(VaulticError::FileNotFound { path });
                    }
                    AgeBackend::new(path)
                }
                None => {
                    if let Ok(key_data) = std::env::var("VAULTIC_AGE_KEY") {
                        let key_data = key_data.trim();
                        if key_data.is_empty() {
                            return Err(VaulticError::EncryptionFailed {
                                reason: "VAULTIC_AGE_KEY is set but empty. Provide the full age identity content.".into(),
                            });
                        }
                        AgeBackend::from_key_data(key_data.to_string())
                    } else {
                        let path = AgeBackend::default_identity_path()?;
                        if !path.exists() {
                            return Err(VaulticError::EncryptionFailed {
                                reason: format!(
                                    "No private key found at {}\n\n  Solutions:\n    \
                                     → New here? Run 'vaultic keys setup' to generate a key\n    \
                                     → Set VAULTIC_AGE_KEY environment variable with your private key\n    \
                                     → Have a key? Use --key <path> to specify the location\n    \
                                     → Lost your key? Ask an admin to re-add you as a recipient",
                                    path.display()
                                ),
                            });
                        }
                        AgeBackend::new(path)
                    }
                }
            };
            decrypt_with(backend, key_store, &source, &dest, env_name, to_stdout)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            if !backend.is_available() {
                return Err(VaulticError::EncryptionFailed {
                    reason: "GPG is not installed or not found in PATH".into(),
                });
            }
            decrypt_with(backend, key_store, &source, &dest, env_name, to_stdout)
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
    to_stdout: bool,
) -> Result<()> {
    let cipher_name = cipher.name().to_string();

    let service = EncryptionService { cipher, key_store };

    if to_stdout {
        let plaintext = service.decrypt_to_bytes(source)?;
        let content = String::from_utf8(plaintext).map_err(|_| VaulticError::ParseError {
            file: source.to_path_buf(),
            detail: "Decrypted content is not valid UTF-8".into(),
        })?;
        print!("{content}");
        return Ok(());
    }

    output::detail(&format!("Source: {}", source.display()));
    output::detail(&format!("Destination: {}", dest.display()));

    let sp = output::spinner(&format!("Decrypting {env_name} with {cipher_name}..."));
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

    output::finish_spinner(sp, &format!("Decrypted {}", source.display()));
    output::success(&format!(
        "Generated {} with {var_count} variables",
        dest.display()
    ));
    println!("\n  Run 'vaultic check' to verify no variables are missing.");

    // Audit
    let state_hash = super::audit_helpers::compute_file_hash(dest);
    super::audit_helpers::log_audit_with_hash(
        crate::core::models::audit_entry::AuditAction::Decrypt,
        vec![format!("{env_name}.env.enc")],
        Some(format!(
            "{var_count} variables decrypted to {}",
            dest.display()
        )),
        state_hash,
    );

    Ok(())
}
