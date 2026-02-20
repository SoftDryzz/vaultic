use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::adapters::cipher::age_backend::AgeBackend;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};

/// Execute the `vaultic init` command.
///
/// Creates the `.vaultic/` directory structure, generates config defaults,
/// and optionally sets up encryption keys via interactive prompts.
pub fn execute(verbose: bool) -> Result<()> {
    let vaultic_dir = Path::new(".vaultic");

    if vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic is already initialized in this project (.vaultic/ exists)".into(),
        });
    }

    output::header("Vaultic — Initializing project");

    // Create directory structure
    std::fs::create_dir_all(vaultic_dir)?;
    output::success("Created .vaultic/");

    // Generate config.toml
    let config_content = r#"[vaultic]
version = "0.1.0"
default_cipher = "age"
default_env = "dev"

[environments]
base = { file = "base.env" }
dev = { file = "dev.env", inherits = "base" }
staging = { file = "staging.env", inherits = "base" }
prod = { file = "prod.env", inherits = "base" }

[audit]
enabled = true
log_file = "audit.log"
"#;
    std::fs::write(vaultic_dir.join("config.toml"), config_content)?;
    output::success("Generated config.toml with defaults");

    // Create empty recipients file
    std::fs::write(vaultic_dir.join("recipients.txt"), "")?;

    // Create .env.template
    if !Path::new(".env.template").exists() {
        std::fs::write(".env.template", "# Add your environment variables here\n")?;
        output::success("Created .env.template");
    }

    // Add .env to .gitignore
    add_to_gitignore(".env")?;

    // Key setup
    output::header("Key configuration");
    println!("  Searching for existing keys...\n");

    let identity_path = AgeBackend::default_identity_path()?;

    if identity_path.exists() {
        let public_key = AgeBackend::read_public_key(&identity_path)?;
        output::success(&format!("Age key found at {}", identity_path.display()));
        output::success(&format!("Public key: {public_key}"));

        add_self_to_recipients(vaultic_dir, &public_key)?;
    } else {
        output::warning("No age or GPG key found\n");
        print!("  Generate a new age key now? [Y/n]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer.is_empty() || answer == "y" || answer == "yes" {
            println!();
            let public_key = AgeBackend::generate_identity(&identity_path)?;
            output::success(&format!(
                "Private key saved to: {}",
                identity_path.display()
            ));
            output::success(&format!("Public key: {public_key}"));

            print_key_warning(&identity_path);
            add_self_to_recipients(vaultic_dir, &public_key)?;
        } else {
            output::warning("Skipped key generation");
            println!("  Run 'vaultic keys setup' later to configure your key.\n");
        }
    }

    output::success("Project ready.\n");
    print_next_steps(verbose);

    Ok(())
}

/// Add an entry to .gitignore if not already present.
fn add_to_gitignore(entry: &str) -> Result<()> {
    let gitignore = Path::new(".gitignore");

    if gitignore.exists() {
        let content = std::fs::read_to_string(gitignore)?;
        if content.lines().any(|l| l.trim() == entry) {
            output::success(&format!("{entry} already in .gitignore"));
            return Ok(());
        }
        let mut file = std::fs::OpenOptions::new().append(true).open(gitignore)?;
        writeln!(file, "\n# Vaultic: never commit plaintext secrets\n{entry}")?;
    } else {
        std::fs::write(
            gitignore,
            format!("# Vaultic: never commit plaintext secrets\n{entry}\n"),
        )?;
    }

    output::success(&format!("Added {entry} to .gitignore"));
    Ok(())
}

/// Add the user's own public key to recipients.txt.
fn add_self_to_recipients(vaultic_dir: &Path, public_key: &str) -> Result<()> {
    let recipients_path = vaultic_dir.join("recipients.txt");
    std::fs::write(&recipients_path, format!("{public_key}\n"))?;
    output::success("Public key added to .vaultic/recipients.txt");
    Ok(())
}

/// Print the private key safety warning box.
fn print_key_warning(identity_path: &Path) {
    use colored::Colorize;

    let location = format!("Location: {}", identity_path.display());
    let pad_len = 55usize.saturating_sub(location.len());
    let padding = " ".repeat(pad_len);

    println!();
    println!(
        "  {}",
        "┌─────────────────────────────────────────────────────────┐".yellow()
    );
    println!(
        "  {}",
        "│ IMPORTANT: About your private key                       │".yellow()
    );
    println!(
        "  {}",
        "│                                                         │".yellow()
    );
    println!("  {} {location}{padding}{}", "│".yellow(), "│".yellow());
    println!(
        "  {}",
        "│                                                         │".yellow()
    );
    println!(
        "  {}",
        "│ • NEVER share this file with anyone                     │".yellow()
    );
    println!(
        "  {}",
        "│ • Back it up somewhere safe (USB, password manager)     │".yellow()
    );
    println!(
        "  {}",
        "│ • If you lose it, you CANNOT decrypt your secrets       │".yellow()
    );
    println!(
        "  {}",
        "│ • Your PUBLIC key IS safe to share                      │".yellow()
    );
    println!(
        "  {}",
        "└─────────────────────────────────────────────────────────┘".yellow()
    );
    println!();
}

/// Print next steps after init.
fn print_next_steps(verbose: bool) {
    println!("  Next steps:");
    println!("     1. Create your .env with the project variables");
    println!("     2. Run 'vaultic encrypt' to encrypt it");
    println!("     3. Commit .vaultic/ to the repo");
    println!("     4. Share your PUBLIC key with the team");

    if verbose {
        println!();
        println!("  Files created:");
        println!("     .vaultic/config.toml      — Vaultic configuration");
        println!("     .vaultic/recipients.txt   — Authorized public keys");
        println!("     .env.template             — Variable template (commit this)");
    }
}
