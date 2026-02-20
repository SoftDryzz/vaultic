use std::path::{Path, PathBuf};

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::cipher::gpg_backend::GpgBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::encryption_service::EncryptionService;
use crate::core::traits::cipher::CipherBackend;
use crate::core::traits::key_store::KeyStore;

/// Execute the `vaultic encrypt` command.
///
/// Encrypts a source file for all authorized recipients
/// and stores the ciphertext in `.vaultic/`.
pub fn execute(file: Option<&str>, env: Option<&str>, cipher: &str) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let source = PathBuf::from(file.unwrap_or(".env"));
    if !source.exists() {
        return Err(VaulticError::FileNotFound {
            path: source.clone(),
        });
    }

    let env_name = env.unwrap_or("dev");
    let dest = vaultic_dir.join(format!("{env_name}.env.enc"));
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            let backend = AgeBackend::new(identity_path);
            encrypt_with(backend, key_store, &source, &dest, env_name)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            if !backend.is_available() {
                return Err(VaulticError::EncryptionFailed {
                    reason: "GPG is not installed or not found in PATH".into(),
                });
            }
            encrypt_with(backend, key_store, &source, &dest, env_name)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Encrypt with a given backend.
fn encrypt_with<C: CipherBackend>(
    cipher: C,
    key_store: FileKeyStore,
    source: &Path,
    dest: &Path,
    env_name: &str,
) -> Result<()> {
    let recipients = key_store.list()?;
    let cipher_name = cipher.name().to_string();

    let service = EncryptionService { cipher, key_store };

    output::header(&format!("Encrypting with {cipher_name} for {env_name}"));

    service.encrypt_file(source, dest)?;

    output::success(&format!(
        "Encrypted with {cipher_name} for {} recipient(s)",
        recipients.len()
    ));
    output::success(&format!("Saved to {}", dest.display()));
    println!("\n  Commit {} to the repo.", dest.display());

    // Audit
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::Encrypt,
        vec![format!("{env_name}.env.enc")],
        Some(format!(
            "encrypted with {cipher_name} for {} recipient(s)",
            recipients.len()
        )),
    );

    Ok(())
}
