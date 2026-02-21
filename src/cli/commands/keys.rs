use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::adapters::cipher::gpg_backend::GpgBackend;
use crate::adapters::key_stores::file_key_store::FileKeyStore;
use crate::cli::KeysAction;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::key_identity::KeyIdentity;
use crate::core::services::key_service::KeyService;

/// Execute the `vaultic keys` command.
pub fn execute(action: &KeysAction) -> Result<()> {
    match action {
        KeysAction::Setup => execute_setup(),
        KeysAction::Add { identity } => execute_add(identity),
        KeysAction::List => execute_list(),
        KeysAction::Remove { identity } => execute_remove(identity),
    }
}

/// Interactive key setup for new users.
fn execute_setup() -> Result<()> {
    output::header("Key configuration for Vaultic");

    let identity_path = AgeBackend::default_identity_path()?;

    if identity_path.exists() {
        let public_key = AgeBackend::read_public_key(&identity_path)?;
        output::success(&format!(
            "Age key already exists at {}",
            identity_path.display()
        ));
        output::success(&format!("Public key: {public_key}"));

        println!("\n  Share this PUBLIC key with the project admin.");
        println!("  The admin will run: vaultic keys add {public_key}");
        return Ok(());
    }

    // Detect GPG availability
    let gpg = GpgBackend::new();
    let gpg_available = gpg.is_available();

    println!("\n  What do you want to do?");
    println!("  1. Generate a new age key (recommended for new users)");
    println!("  2. Import an existing age key from file");
    if gpg_available {
        println!("  3. Use an existing GPG key from your keyring");
    }
    println!();
    print!("  Selection [1]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "" | "1" => setup_generate_age(&identity_path)?,
        "2" => setup_import_age(&identity_path)?,
        "3" if gpg_available => setup_use_gpg()?,
        _ => {
            println!(
                "\n  When you have your key ready, share the public key with the project admin."
            );
        }
    }

    Ok(())
}

/// Option 1: Generate a new age key.
fn setup_generate_age(identity_path: &Path) -> Result<()> {
    println!();
    let public_key = AgeBackend::generate_identity(identity_path)?;
    output::success(&format!("Private key: {}", identity_path.display()));
    output::success(&format!("Public key: {public_key}"));

    print_next_step(&public_key);
    try_auto_add_recipient(&public_key);
    Ok(())
}

/// Option 2: Import an existing age key from a file.
fn setup_import_age(identity_path: &Path) -> Result<()> {
    print!("\n  Path to your age identity file: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let source = PathBuf::from(input.trim());

    if !source.exists() {
        return Err(VaulticError::FileNotFound { path: source });
    }

    // Validate that the file contains a valid age identity
    let public_key =
        AgeBackend::read_public_key(&source).map_err(|_| VaulticError::InvalidConfig {
            detail: format!(
                "File does not contain a valid age identity: {}\n\n  \
                 Expected a file with an AGE-SECRET-KEY-... line.",
                source.display()
            ),
        })?;

    // Copy the identity file to the default location
    if let Some(parent) = identity_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&source, identity_path)?;

    output::success(&format!("Key imported to {}", identity_path.display()));
    output::success(&format!("Public key: {public_key}"));

    print_next_step(&public_key);
    try_auto_add_recipient(&public_key);
    Ok(())
}

/// Option 3: Use an existing GPG key from the system keyring.
fn setup_use_gpg() -> Result<()> {
    // List available GPG secret keys
    let list_output = std::process::Command::new("gpg")
        .args(["--list-secret-keys", "--keyid-format", "long"])
        .output()
        .map_err(|e| VaulticError::EncryptionFailed {
            reason: format!("Failed to list GPG keys: {e}"),
        })?;

    if !list_output.status.success() {
        return Err(VaulticError::EncryptionFailed {
            reason: "Failed to list GPG secret keys".into(),
        });
    }

    let key_list = String::from_utf8_lossy(&list_output.stdout);
    println!("\n  Available GPG keys:\n");
    for line in key_list.lines() {
        println!("  {line}");
    }

    print!("\n  Enter the GPG key ID or email to use: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let gpg_id = input.trim().to_string();

    if gpg_id.is_empty() {
        output::warning("No key selected, setup skipped.");
        return Ok(());
    }

    output::success(&format!("GPG key selected: {gpg_id}"));
    println!("\n  Use --cipher gpg when encrypting/decrypting.");

    print_next_step(&gpg_id);
    try_auto_add_recipient(&gpg_id);
    Ok(())
}

/// Print next step instructions after key setup.
fn print_next_step(public_key: &str) {
    println!();
    println!("  Next step:");
    println!("  Send your PUBLIC key to the project admin:");
    println!("  {public_key}");
    println!();
    println!("  The admin will run:");
    println!("  vaultic keys add {public_key}");
    println!();
    println!("  After that you can decrypt with: vaultic decrypt --env dev");
}

/// Try to auto-add the public key to recipients if .vaultic exists.
fn try_auto_add_recipient(public_key: &str) {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    let recipients_path = vaultic_dir.join("recipients.txt");
    if recipients_path.exists() {
        let store = FileKeyStore::new(recipients_path);
        let service = KeyService { store };
        let ki = KeyIdentity {
            public_key: public_key.to_string(),
            label: None,
            added_at: Some(chrono::Utc::now()),
        };
        if service.add_key(&ki).is_ok() {
            output::success("Public key added to .vaultic/recipients.txt");
        }
    }
}

/// Add a recipient public key.
fn execute_add(identity: &str) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));
    let service = KeyService { store };

    let ki = KeyIdentity {
        public_key: identity.to_string(),
        label: None,
        added_at: Some(chrono::Utc::now()),
    };

    service.add_key(&ki)?;
    output::success(&format!("Added recipient: {identity}"));
    println!("\n  Re-encrypt with 'vaultic encrypt' so this recipient can decrypt.");

    // Audit
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::KeyAdd,
        vec![],
        Some(format!("added {identity}")),
    );

    Ok(())
}

/// List all authorized recipients.
fn execute_list() -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));
    let service = KeyService { store };
    let keys = service.list_keys()?;

    if keys.is_empty() {
        output::warning("No recipients configured.");
        println!("  Run 'vaultic keys add <public-key>' to add one.");
        return Ok(());
    }

    output::header(&format!("Authorized recipients ({})", keys.len()));
    for ki in &keys {
        match &ki.label {
            Some(label) => println!("  • {}  # {label}", ki.public_key),
            None => println!("  • {}", ki.public_key),
        }
    }

    Ok(())
}

/// Remove a recipient by public key.
fn execute_remove(identity: &str) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    let store = FileKeyStore::new(vaultic_dir.join("recipients.txt"));
    let service = KeyService { store };

    service.remove_key(identity)?;
    output::success(&format!("Removed recipient: {identity}"));
    println!("\n  Re-encrypt with 'vaultic encrypt --all' to revoke this recipient's access.");

    // Audit
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::KeyRemove,
        vec![],
        Some(format!("removed {identity}")),
    );

    Ok(())
}
