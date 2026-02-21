use std::sync::OnceLock;

use colored::Colorize;

/// Verbosity level for CLI output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

static VERBOSITY: OnceLock<Verbosity> = OnceLock::new();

/// Initialize the global verbosity level. Must be called once at startup.
pub fn init(verbose: bool, quiet: bool) {
    let level = if quiet {
        Verbosity::Quiet
    } else if verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };
    let _ = VERBOSITY.set(level);
}

/// Get the current verbosity level.
fn verbosity() -> Verbosity {
    VERBOSITY.get().copied().unwrap_or(Verbosity::Normal)
}

/// Print a success message (suppressed in quiet mode).
pub fn success(msg: &str) {
    if verbosity() != Verbosity::Quiet {
        println!("  {} {}", "✓".green(), msg);
    }
}

/// Print a warning message (suppressed in quiet mode).
pub fn warning(msg: &str) {
    if verbosity() != Verbosity::Quiet {
        println!("  {} {}", "⚠".yellow(), msg);
    }
}

/// Print an error message (always shown).
pub fn error(msg: &str) {
    eprintln!("  {} {}", "✗".red(), msg);
}

/// Print a header line (suppressed in quiet mode).
pub fn header(msg: &str) {
    if verbosity() != Verbosity::Quiet {
        println!("\n{}", msg.bold());
    }
}

/// Print a detail message (only shown in verbose mode).
pub fn detail(msg: &str) {
    if verbosity() == Verbosity::Verbose {
        println!("  {} {}", "·".dimmed(), msg);
    }
}
