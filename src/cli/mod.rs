pub mod commands;
pub mod context;
pub mod output;

use clap::{Parser, Subcommand};

/// Secure your secrets. Sync your team. Trust your configs.
#[derive(Parser, Debug)]
#[command(
    name = "vaultic",
    version,
    about,
    long_about = "Vaultic is a CLI tool for managing secrets and configuration files \
                  securely across development teams.\n\n\
                  It encrypts your sensitive files with age or GPG, syncs them via Git, \
                  detects missing variables, supports multi-environment inheritance, \
                  and audits every change.",
    after_help = "Getting started:\n  \
                  New project:     vaultic init\n  \
                  Join a project:  vaultic keys setup → send your public key to admin\n  \
                  Check status:    vaultic status\n\n\
                  More info: https://github.com/SoftDryzz/vaultic"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Encryption backend to use
    #[arg(long, global = true, default_value = "age")]
    pub cipher: String,

    /// Target environment(s). Repeat for diff: --env dev --env prod
    #[arg(long, global = true)]
    pub env: Vec<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode: only show errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Path to alternative config file
    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Vaultic in the current project
    #[command(
        long_about = "Initialize Vaultic in the current project.\n\n\
                      Creates the .vaultic/ directory, generates config.toml with defaults, \
                      creates an empty .env.template, and adds .env to .gitignore.\n\n\
                      During setup, Vaultic detects existing age and GPG keys and offers \
                      to generate a new key if none is found.",
        after_help = "Examples:\n  \
                      vaultic init              # Interactive setup with key detection\n  \
                      vaultic init --cipher gpg # Initialize with GPG as default backend"
    )]
    Init,

    /// Encrypt secret files
    #[command(
        long_about = "Encrypt secret files for all authorized recipients.\n\n\
                      Reads the source file, encrypts it with the public keys of all \
                      recipients listed in .vaultic/recipients.txt, and saves the \
                      ciphertext as .vaultic/<env>.env.enc.\n\n\
                      The original file is NOT modified or deleted. Use --all to \
                      re-encrypt all environments (useful after adding/removing recipients).",
        after_help = "Examples:\n  \
                      vaultic encrypt                       # Encrypt .env as dev\n  \
                      vaultic encrypt .env --env prod       # Encrypt as prod environment\n  \
                      vaultic encrypt --all                 # Re-encrypt all environments\n  \
                      vaultic encrypt --cipher gpg          # Encrypt with GPG backend"
    )]
    Encrypt {
        /// File to encrypt (default: .env)
        file: Option<String>,
        /// Re-encrypt all environments for current recipients
        #[arg(long)]
        all: bool,
    },

    /// Decrypt secret files
    #[command(
        long_about = "Decrypt secret files using your private key.\n\n\
                      Reads the encrypted file from .vaultic/<env>.env.enc and writes \
                      the plaintext to .env in the working directory (by default).\n\n\
                      Use --output to write the decrypted file to a custom path. \
                      This is useful when running Vaultic from a parent directory \
                      but the application expects .env in a subdirectory.\n\n\
                      By default, uses the age key at ~/.config/age/keys.txt. \
                      Use --key to specify a different private key location.",
        after_help = "Examples:\n  \
                      vaultic decrypt                       # Decrypt dev → ./.env\n  \
                      vaultic decrypt --env prod            # Decrypt prod → ./.env\n  \
                      vaultic decrypt -o backend/.env       # Decrypt dev → backend/.env\n  \
                      vaultic decrypt --key /path/to/key    # Use custom private key\n  \
                      vaultic decrypt --cipher gpg          # Decrypt with GPG backend"
    )]
    Decrypt {
        /// File to decrypt
        file: Option<String>,
        /// Path to private key file
        #[arg(long)]
        key: Option<String>,
        /// Output path for the decrypted file (default: .env)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Verify missing variables against template
    #[command(
        long_about = "Verify your local .env against .env.template.\n\n\
                      Reports missing variables (in template but not in .env), \
                      extra variables (in .env but not in template), and \
                      variables with empty values.",
        after_help = "Examples:\n  \
                      vaultic check                         # Check .env vs .env.template"
    )]
    Check,

    /// Compare secret files or environments
    #[command(
        long_about = "Compare two secret files or two resolved environments side by side.\n\n\
                      In file mode, compares two .env files directly.\n\
                      In environment mode (--env dev --env prod), resolves the full \
                      inheritance chain for each environment before comparing.",
        after_help = "Examples:\n  \
                      vaultic diff .env .env.prod           # Compare two files\n  \
                      vaultic diff --env dev --env prod     # Compare resolved environments\n  \
                      vaultic diff --env dev --env prod --cipher gpg"
    )]
    Diff {
        /// First file to compare
        file1: Option<String>,
        /// Second file to compare
        file2: Option<String>,
    },

    /// Generate resolved file with inheritance applied
    #[command(
        long_about = "Generate a resolved .env file by applying environment inheritance.\n\n\
                      Reads the inheritance chain from .vaultic/config.toml, decrypts \
                      each layer in memory, and merges them from base to leaf. \
                      The overlay always wins when keys conflict.\n\n\
                      Use --output to write the resolved file to a custom path instead \
                      of the default .env in the working directory.",
        after_help = "Examples:\n  \
                      vaultic resolve --env dev             # Resolve dev → ./.env\n  \
                      vaultic resolve --env staging         # Resolve staging chain\n  \
                      vaultic resolve --env prod -o prod.env  # Resolve prod → prod.env\n  \
                      vaultic resolve --env prod --cipher gpg"
    )]
    Resolve {
        /// Output path for the resolved file (default: .env)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Manage keys and recipients
    #[command(
        long_about = "Manage encryption keys and authorized recipients.\n\n\
                      Recipients are public keys stored in .vaultic/recipients.txt. \
                      Only recipients can decrypt files encrypted for the project.",
        after_help = "Examples:\n  \
                      vaultic keys setup                    # Generate or import a key\n  \
                      vaultic keys add age1abc...xyz        # Add a recipient\n  \
                      vaultic keys list                     # List all recipients\n  \
                      vaultic keys remove age1abc...xyz     # Remove a recipient"
    )]
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },

    /// Show operation history
    #[command(
        long_about = "Show the audit log of all Vaultic operations.\n\n\
                      Each entry records the timestamp, author (from git config), \
                      action performed, affected files, and an optional state hash.",
        after_help = "Examples:\n  \
                      vaultic log                           # Show full history\n  \
                      vaultic log --last 10                 # Show last 10 entries\n  \
                      vaultic log --author \"Alice\"          # Filter by author\n  \
                      vaultic log --since 2026-01-01        # Filter by date"
    )]
    Log {
        /// Filter by author
        #[arg(long)]
        author: Option<String>,
        /// Filter entries since this date (ISO 8601)
        #[arg(long)]
        since: Option<String>,
        /// Show last N entries
        #[arg(long)]
        last: Option<usize>,
    },

    /// Show full project status
    #[command(long_about = "Show a full project dashboard.\n\n\
                      Displays configuration, authorized recipients, encrypted \
                      environments with file sizes, local state (.env, template, \
                      gitignore), your key info, and audit log entry count.")]
    Status,

    /// Install or uninstall git hooks
    #[command(
        long_about = "Manage git hooks for secret safety.\n\n\
                      The pre-commit hook blocks plaintext .env files from being \
                      committed accidentally. It detects Vaultic-managed hooks via \
                      marker comments and refuses to overwrite foreign hooks.",
        after_help = "Examples:\n  \
                      vaultic hook install                  # Install pre-commit hook\n  \
                      vaultic hook uninstall                # Remove pre-commit hook"
    )]
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Update Vaultic to the latest version
    #[command(
        long_about = "Check for and install the latest Vaultic release.\n\n\
                      Downloads the binary for your platform from GitHub Releases, \
                      verifies its SHA256 checksum and minisign cryptographic signature, \
                      then replaces the running binary.\n\n\
                      The update is safe: your encrypted files and configuration are \
                      never modified. Only the vaultic binary itself is replaced.",
        after_help = "Examples:\n  \
                      vaultic update                        # Check and install latest version"
    )]
    Update,
}

#[derive(Subcommand, Debug)]
pub enum KeysAction {
    /// Generate or import a key for this project
    #[command(long_about = "Interactive key setup for new users.\n\n\
                      Options:\n  \
                      1. Generate a new age key (recommended)\n  \
                      2. Import an existing age key from file\n  \
                      3. Use an existing GPG key from the system keyring")]
    Setup,
    /// Add a recipient (public key)
    #[command(after_help = "Accepted formats:\n  \
                            age key:          age1ql3z7hjy54pw...ac8p\n  \
                            GPG fingerprint:  A1B2C3D4E5F6...\n  \
                            GPG email:        user@example.com")]
    Add {
        /// Public key or identity to add
        identity: String,
    },
    /// List authorized recipients
    List,
    /// Remove a recipient
    Remove {
        /// Public key or identity to remove
        identity: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum HookAction {
    /// Install git pre-commit hook
    Install,
    /// Uninstall git pre-commit hook
    Uninstall,
}
