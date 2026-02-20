pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};

/// Secure your secrets. Sync your team. Trust your configs.
#[derive(Parser, Debug)]
#[command(name = "vaultic", version, about, long_about = None)]
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
    Init,

    /// Encrypt secret files
    Encrypt {
        /// File to encrypt (default: .env)
        file: Option<String>,
    },

    /// Decrypt secret files
    Decrypt {
        /// File to decrypt
        file: Option<String>,
    },

    /// Verify missing variables against template
    Check,

    /// Compare secret files or environments
    Diff {
        /// First file to compare
        file1: Option<String>,
        /// Second file to compare
        file2: Option<String>,
    },

    /// Generate resolved file with inheritance applied
    Resolve,

    /// Manage keys and recipients
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },

    /// Show operation history
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
    Status,

    /// Install or uninstall git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeysAction {
    /// Generate or import a key for this project
    Setup,
    /// Add a recipient (public key)
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
