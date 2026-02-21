use std::path::Path;

use crate::adapters::git::git_hook;
use crate::cli::HookAction;
use crate::cli::output;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditAction;

/// Execute the `vaultic hook` command.
pub fn execute(action: &HookAction) -> Result<()> {
    match action {
        HookAction::Install => execute_install(),
        HookAction::Uninstall => execute_uninstall(),
    }
}

/// Install the git pre-commit hook.
fn execute_install() -> Result<()> {
    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        return Err(VaulticError::HookError {
            detail: "Not a git repository. Run 'git init' first.".into(),
        });
    }

    output::header("Installing git pre-commit hook");

    git_hook::install(git_dir)?;

    output::success("Pre-commit hook installed at .git/hooks/pre-commit");
    println!("\n  The hook will block commits that include plaintext .env files.");
    println!("  To remove it later: vaultic hook uninstall");

    super::audit_helpers::log_audit(AuditAction::HookInstall, vec![], None);

    Ok(())
}

/// Uninstall the git pre-commit hook.
fn execute_uninstall() -> Result<()> {
    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        return Err(VaulticError::HookError {
            detail: "Not a git repository.".into(),
        });
    }

    output::header("Uninstalling git pre-commit hook");

    git_hook::uninstall(git_dir)?;

    output::success("Pre-commit hook removed");

    super::audit_helpers::log_audit(AuditAction::HookUninstall, vec![], None);

    Ok(())
}
