use std::fs;
use std::path::Path;

use crate::core::errors::{Result, VaulticError};

/// Marker comment used to identify Vaultic-managed hooks.
const HOOK_MARKER: &str = "# vaultic-managed-hook";

/// The pre-commit hook script that prevents committing plaintext secrets.
///
/// The hook checks staged files for patterns that indicate secrets
/// (e.g. `.env` without `.enc`) and blocks the commit with a clear message.
const PRE_COMMIT_SCRIPT: &str = r#"#!/bin/sh
# vaultic-managed-hook
# Vaultic pre-commit hook — blocks plaintext secrets from being committed.
# Installed by: vaultic hook install
# Remove with:  vaultic hook uninstall

staged=$(git diff --cached --name-only)

blocked=""
for file in $staged; do
    case "$file" in
        .env|.env.*)
            # Allow .env.template and .env.example
            case "$file" in
                *.template|*.example) ;;
                *.enc) ;;
                *) blocked="$blocked $file" ;;
            esac
            ;;
    esac
done

if [ -n "$blocked" ]; then
    echo ""
    echo "  STOP — Vaultic pre-commit hook"
    echo ""
    echo "  Plaintext secret files staged for commit:"
    for f in $blocked; do
        echo "    - $f"
    done
    echo ""
    echo "  These files contain sensitive data and should NOT be committed."
    echo ""
    echo "  Solutions:"
    echo "    -> Encrypt first: vaultic encrypt"
    echo "    -> Or unstage:    git reset HEAD $blocked"
    echo "    -> Skip check:    git commit --no-verify (NOT recommended)"
    echo ""
    exit 1
fi
"#;

/// Install the Vaultic pre-commit hook.
///
/// If a pre-commit hook already exists and is not managed by Vaultic,
/// returns an error to avoid overwriting user hooks.
pub fn install(git_dir: &Path) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)?;
    }

    let hook_path = hooks_dir.join("pre-commit");

    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path)?;
        if !content.contains(HOOK_MARKER) {
            return Err(VaulticError::HookError {
                detail: format!(
                    "A pre-commit hook already exists at {}\n\n  \
                     It was not installed by Vaultic and will not be overwritten.\n  \
                     To replace it, remove the existing hook first:\n  \
                     rm {}",
                    hook_path.display(),
                    hook_path.display()
                ),
            });
        }
    }

    fs::write(&hook_path, PRE_COMMIT_SCRIPT)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

/// Uninstall the Vaultic pre-commit hook.
///
/// Only removes the hook if it was installed by Vaultic (contains the marker).
pub fn uninstall(git_dir: &Path) -> Result<()> {
    let hook_path = git_dir.join("hooks").join("pre-commit");

    if !hook_path.exists() {
        return Err(VaulticError::HookError {
            detail: "No pre-commit hook found. Nothing to uninstall.".into(),
        });
    }

    let content = fs::read_to_string(&hook_path)?;
    if !content.contains(HOOK_MARKER) {
        return Err(VaulticError::HookError {
            detail: "The pre-commit hook was not installed by Vaultic. Not removing it.".into(),
        });
    }

    fs::remove_file(&hook_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_git_dir() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("hooks")).unwrap();
        tmp
    }

    #[test]
    fn install_creates_hook() {
        let git_dir = setup_git_dir();
        install(git_dir.path()).unwrap();

        let hook = git_dir.path().join("hooks/pre-commit");
        assert!(hook.exists());

        let content = fs::read_to_string(hook).unwrap();
        assert!(content.contains(HOOK_MARKER));
        assert!(content.contains("git diff --cached"));
    }

    #[test]
    fn install_overwrites_vaultic_hook() {
        let git_dir = setup_git_dir();
        install(git_dir.path()).unwrap();

        // Install again — should succeed (same marker)
        install(git_dir.path()).unwrap();
    }

    #[test]
    fn install_refuses_foreign_hook() {
        let git_dir = setup_git_dir();
        let hook_path = git_dir.path().join("hooks/pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho custom hook\n").unwrap();

        let result = install(git_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_removes_vaultic_hook() {
        let git_dir = setup_git_dir();
        install(git_dir.path()).unwrap();
        uninstall(git_dir.path()).unwrap();

        assert!(!git_dir.path().join("hooks/pre-commit").exists());
    }

    #[test]
    fn uninstall_refuses_foreign_hook() {
        let git_dir = setup_git_dir();
        let hook_path = git_dir.path().join("hooks/pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho custom\n").unwrap();

        let result = uninstall(git_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_no_hook_fails() {
        let git_dir = setup_git_dir();
        let result = uninstall(git_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn install_creates_hooks_dir_if_missing() {
        let tmp = TempDir::new().unwrap();
        // No hooks dir exists
        install(tmp.path()).unwrap();

        assert!(tmp.path().join("hooks/pre-commit").exists());
    }
}
