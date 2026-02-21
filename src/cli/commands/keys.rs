use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::adapters::cipher::age_backend::AgeBackend;
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

    println!("\n  What do you want to do?");
    println!("  1. Generate a new age key (recommended for new users)");
    println!("  2. Skip — I'll configure my key manually\n");
    print!("  Selection [1]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let choice = input.trim();

    if choice.is_empty() || choice == "1" {
        println!();
        let public_key = AgeBackend::generate_identity(&identity_path)?;
        output::success(&format!("Private key: {}", identity_path.display()));
        output::success(&format!("Public key: {public_key}"));

        println!();
        println!("  Next step:");
        println!("  Send your PUBLIC key to the project admin:");
        println!("  {public_key}");
        println!();
        println!("  The admin will run:");
        println!("  vaultic keys add {public_key}");
        println!();
        println!("  After that you can decrypt with: vaultic decrypt --env dev");

        // Auto-add to recipients if .vaultic exists
        let recipients_path = Path::new(".vaultic/recipients.txt");
        if recipients_path.exists() {
            let store = FileKeyStore::new(recipients_path.to_path_buf());
            let service = KeyService { store };
            let ki = KeyIdentity {
                public_key: public_key.clone(),
                label: None,
                added_at: Some(chrono::Utc::now()),
            };
            if service.add_key(&ki).is_ok() {
                output::success("Public key added to .vaultic/recipients.txt");
            }
        }
    } else {
        println!("\n  When you have your key ready, share the public key with the project admin.");
    }

    Ok(())
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
