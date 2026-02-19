use colored::Colorize;

/// Print a success message.
pub fn success(msg: &str) {
    println!("  {} {}", "✓".green(), msg);
}

/// Print a warning message.
pub fn warning(msg: &str) {
    println!("  {} {}", "⚠".yellow(), msg);
}

/// Print an error message.
pub fn error(msg: &str) {
    eprintln!("  {} {}", "✗".red(), msg);
}

/// Print a header line.
pub fn header(msg: &str) {
    println!("\n{}", msg.bold());
}
