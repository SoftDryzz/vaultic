use std::path::{Path, PathBuf};

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::cipher::gpg_backend::GpgBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::encryption_service::EncryptionService;
use crate::core::traits::cipher::CipherBackend;
use crate::core::traits::key_store::KeyStore;

/// Execute the `vaultic encrypt` command.
///
/// Encrypts a source file for all authorized recipients
/// and stores the ciphertext in `.vaultic/`.
/// When `all` is true, re-encrypts every environment defined in config.
pub fn execute(file: Option<&str>, env: Option<&str>, cipher: &str, all: bool) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    if all {
        return encrypt_all(vaultic_dir, cipher);
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

    encrypt_single(&source, &dest, env_name, cipher, &key_store)
}

/// Re-encrypt all environments defined in config.toml.
///
/// For each environment, decrypts the existing `.enc` file and
/// re-encrypts it with the current recipients list.
fn encrypt_all(vaultic_dir: &Path, cipher: &str) -> Result<()> {
    let config = AppConfig::load(vaultic_dir)?;
    let key_store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));

    let mut envs: Vec<_> = config.environments.keys().collect();
    envs.sort();

    let mut success_count = 0;
    let mut skip_count = 0;

    for env_name in &envs {
        let file_name = config.env_file_name(env_name);
        let enc_path = vaultic_dir.join(format!("{file_name}.enc"));

        if !enc_path.exists() {
            output::warning(&format!("Skipping {env_name}: {file_name}.enc not found"));
            skip_count += 1;
            continue;
        }

        // Decrypt in memory and re-encrypt directly — no plaintext on disk
        let ciphertext = std::fs::read(&enc_path)?;
        let plaintext = decrypt_bytes(&ciphertext, cipher)?;

        encrypt_bytes_to(&plaintext, &enc_path, env_name, cipher, &key_store)?;

        success_count += 1;
    }

    output::success(&format!(
        "Re-encrypted {success_count} environment(s), skipped {skip_count}"
    ));

    Ok(())
}

/// Decrypt raw bytes using the specified cipher backend.
fn decrypt_bytes(ciphertext: &[u8], cipher: &str) -> Result<Vec<u8>> {
    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            let backend = AgeBackend::new(identity_path);
            backend.decrypt(ciphertext)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            backend.decrypt(ciphertext)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Encrypt a single file for one environment.
fn encrypt_single(
    source: &Path,
    dest: &Path,
    env_name: &str,
    cipher: &str,
    key_store: &FileKeyStore,
) -> Result<()> {
    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            let backend = AgeBackend::new(identity_path);
            encrypt_with(backend, key_store, source, dest, env_name)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            if !backend.is_available() {
                return Err(VaulticError::EncryptionFailed {
                    reason: "GPG is not installed or not found in PATH".into(),
                });
            }
            encrypt_with(backend, key_store, source, dest, env_name)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Encrypt with a given backend (reads plaintext from file).
fn encrypt_with<C: CipherBackend>(
    cipher: C,
    key_store: &FileKeyStore,
    source: &Path,
    dest: &Path,
    env_name: &str,
) -> Result<()> {
    let recipients = key_store.list()?;
    let cipher_name = cipher.name().to_string();

    let service = EncryptionService {
        cipher,
        key_store: key_store.clone(),
    };

    output::detail(&format!("Source: {}", source.display()));
    for r in &recipients {
        output::detail(&format!("Recipient: {}", r.public_key));
    }

    let sp = output::spinner(&format!(
        "Encrypting {env_name} with {cipher_name} for {} recipient(s)...",
        recipients.len()
    ));
    service.encrypt_file(source, dest)?;
    output::finish_spinner(
        sp,
        &format!(
            "Encrypted with {cipher_name} for {} recipient(s)",
            recipients.len()
        ),
    );

    output::success(&format!("Saved to {}", dest.display()));
    println!("\n  Commit {} to the repo.", dest.display());

    log_encrypt_audit(env_name, &cipher_name, recipients.len(), dest);

    Ok(())
}

/// Encrypt from in-memory bytes (no plaintext written to disk).
///
/// Used by `encrypt --all` to re-encrypt already-decrypted content
/// without ever writing plaintext to a temp file.
fn encrypt_bytes_to(
    plaintext: &[u8],
    dest: &Path,
    env_name: &str,
    cipher: &str,
    key_store: &FileKeyStore,
) -> Result<()> {
    match cipher {
        "age" => {
            let identity_path = AgeBackend::default_identity_path()?;
            let backend = AgeBackend::new(identity_path);
            encrypt_bytes_with(backend, key_store, plaintext, dest, env_name)
        }
        "gpg" => {
            let backend = GpgBackend::new();
            encrypt_bytes_with(backend, key_store, plaintext, dest, env_name)
        }
        other => Err(VaulticError::InvalidConfig {
            detail: format!("Unknown cipher backend: '{other}'. Use 'age' or 'gpg'."),
        }),
    }
}

/// Encrypt bytes with a given backend (no file I/O for plaintext).
fn encrypt_bytes_with<C: CipherBackend>(
    cipher: C,
    key_store: &FileKeyStore,
    plaintext: &[u8],
    dest: &Path,
    env_name: &str,
) -> Result<()> {
    let recipients = key_store.list()?;
    let cipher_name = cipher.name().to_string();

    let service = EncryptionService {
        cipher,
        key_store: key_store.clone(),
    };

    let sp = output::spinner(&format!(
        "Re-encrypting {env_name} with {cipher_name} for {} recipient(s)...",
        recipients.len()
    ));
    service.encrypt_bytes(plaintext, dest)?;
    output::finish_spinner(
        sp,
        &format!(
            "Re-encrypted {env_name} with {cipher_name} for {} recipient(s)",
            recipients.len()
        ),
    );

    log_encrypt_audit(env_name, &cipher_name, recipients.len(), dest);

    Ok(())
}

/// Log an encrypt audit entry.
fn log_encrypt_audit(env_name: &str, cipher_name: &str, recipient_count: usize, dest: &Path) {
    let state_hash = super::audit_helpers::compute_file_hash(dest);
    super::audit_helpers::log_audit_with_hash(
        crate::core::models::audit_entry::AuditAction::Encrypt,
        vec![format!("{env_name}.env.enc")],
        Some(format!(
            "encrypted with {cipher_name} for {recipient_count} recipient(s)",
        )),
        state_hash,
    );
}
