use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::commands::crypto_helpers;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::models::audit_entry::AuditAction;
use crate::core::services::env_resolver::EnvResolver;

/// Execute `vaultic ci export`.
///
/// Resolves the environment, then prints secrets to stdout in the
/// requested CI format. No files are written to disk.
pub fn execute_export(env: Option<&str>, cipher: &str, format: &str, mask: bool) -> Result<()> {
    let vaultic_dir = crate::cli::context::vaultic_dir();
    if !vaultic_dir.exists() {
        return Err(VaulticError::InvalidConfig {
            detail: "Vaultic not initialized. Run 'vaultic init' first.".into(),
        });
    }

    // Validate format
    if !matches!(format, "github" | "gitlab" | "generic") {
        return Err(VaulticError::CiExportFailed {
            format: format.to_string(),
        });
    }

    // --mask only makes sense with github format
    if mask && format != "github" {
        return Err(VaulticError::InvalidConfig {
            detail: "--mask is only supported with --format github".into(),
        });
    }

    let config = AppConfig::load(vaultic_dir)?;
    let env_name = env.unwrap_or(&config.vaultic.default_env);
    let parser = DotenvParser;
    let resolver = EnvResolver;

    // Build inheritance chain and decrypt layers
    let chain = resolver.build_chain(env_name, &config)?;
    let files = crypto_helpers::load_env_files(&chain, vaultic_dir, cipher, &parser, false)?;
    let environment = resolver.resolve(env_name, &config, &files)?;

    // Extract key-value pairs from resolved environment.
    let entries: Vec<(&str, &str)> = environment
        .resolved
        .entries()
        .map(|e| (e.key.as_str(), e.value.as_str()))
        .collect();

    // Reject keys that are not valid shell identifiers before printing
    // anything: the output is meant to be eval'd or sourced.
    if let Some((key, _)) = entries.iter().find(|(k, _)| !is_valid_env_key(k)) {
        return Err(VaulticError::InvalidConfig {
            detail: format!(
                "Invalid variable name '{key}' in environment '{env_name}'.\n\n  \
                 CI export only accepts names matching [A-Za-z_][A-Za-z0-9_]*."
            ),
        });
    }

    // Format and print to stdout
    let mut out = String::new();
    for (key, value) in &entries {
        match format {
            "github" => out.push_str(&format_github(key, value, mask)),
            "gitlab" => out.push_str(&format_gitlab(key, value)),
            "generic" => out.push_str(&format_generic(key, value)),
            _ => unreachable!(),
        }
    }
    print!("{out}");

    // Audit (non-blocking)
    super::audit_helpers::log_audit(
        AuditAction::CiExport,
        vec![env_name.to_string()],
        Some(format!("{} variables exported as {format}", entries.len())),
    );

    Ok(())
}

/// Return `true` if `key` is a valid POSIX environment variable name.
fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Quote `s` for POSIX shells. Inside single quotes nothing is expanded,
/// so the only character to handle is the single quote itself.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// A heredoc delimiter for `$GITHUB_ENV` that does not appear as a line
/// of `value`, so a crafted value cannot close the block early and
/// inject additional variables.
fn github_delimiter(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = format!("{:x}", Sha256::digest(value.as_bytes()));
    let mut delim = format!("VAULTIC_EOF_{}", &hash[..16]);
    while value.lines().any(|l| l == delim) {
        delim.push('_');
    }
    delim
}

/// Shell commands that add `key` to `$GITHUB_ENV` (and optionally mask it).
///
/// Values are passed as single-quoted `printf` arguments, so `$()`,
/// backticks and quotes in secrets are never interpreted by the shell.
/// Multi-line values use GitHub's `KEY<<DELIM` syntax.
fn format_github(key: &str, value: &str, mask: bool) -> String {
    let mut out = String::new();
    if mask {
        // ::add-mask:: works per line, so mask every line of the value.
        for line in value.lines().filter(|l| !l.trim().is_empty()) {
            out.push_str(&format!(
                "printf '%s\\n' {}\n",
                shell_quote(&format!("::add-mask::{line}"))
            ));
        }
    }
    if value.contains('\n') || value.contains('\r') {
        let delim = github_delimiter(value);
        out.push_str(&format!(
            "printf '%s\\n' {} {} {} >> \"$GITHUB_ENV\"\n",
            shell_quote(&format!("{key}<<{delim}")),
            shell_quote(value),
            shell_quote(&delim)
        ));
    } else {
        out.push_str(&format!(
            "printf '%s\\n' {} >> \"$GITHUB_ENV\"\n",
            shell_quote(&format!("{key}={value}"))
        ));
    }
    out
}

/// `export KEY='value'` for GitLab CI (or any POSIX shell).
fn format_gitlab(key: &str, value: &str) -> String {
    format!("export {key}={}\n", shell_quote(value))
}

/// `KEY=value` in dotenv format. Values that are not plain are quoted so
/// that `#`, whitespace, `$` and newlines survive a round trip.
fn format_generic(key: &str, value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-.,:/@+=%~^".contains(c));
    if value.is_empty() || plain {
        format!("{key}={value}\n")
    } else if !value.contains('\'') && !value.contains('\n') && !value.contains('\r') {
        format!("{key}='{value}'\n")
    } else {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
            .replace('\r', "\\r")
            .replace('\n', "\\n");
        format!("{key}=\"{escaped}\"\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the generated script with `sh` and return what it appended
    /// to `$GITHUB_ENV` plus its stdout.
    #[cfg(unix)]
    fn run_github_script(script: &str) -> (String, String) {
        let dir = tempfile::tempdir().unwrap();
        let env_file = dir.path().join("github_env");
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("GITHUB_ENV", &env_file)
            .output()
            .unwrap();
        assert!(out.status.success(), "script failed: {script}");
        (
            std::fs::read_to_string(&env_file).unwrap_or_default(),
            String::from_utf8(out.stdout).unwrap(),
        )
    }

    #[test]
    fn valid_env_keys() {
        assert!(is_valid_env_key("API_KEY"));
        assert!(is_valid_env_key("_private"));
        assert!(is_valid_env_key("A1"));
        assert!(!is_valid_env_key("1A"));
        assert!(!is_valid_env_key(""));
        assert!(!is_valid_env_key("A;rm -rf ~;B"));
        assert!(!is_valid_env_key("A B"));
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("a'b"), r"'a'\''b'");
        assert_eq!(shell_quote("$(id)"), "'$(id)'");
    }

    #[test]
    fn github_simple_value() {
        assert_eq!(
            format_github("API_KEY", "secret123", false),
            "printf '%s\\n' 'API_KEY=secret123' >> \"$GITHUB_ENV\"\n"
        );
    }

    #[test]
    fn github_mask_masks_every_line() {
        let out = format_github("PEM", "line1\nline2", true);
        assert!(out.contains("'::add-mask::line1'"));
        assert!(out.contains("'::add-mask::line2'"));
    }

    #[cfg(unix)]
    #[test]
    fn github_does_not_execute_command_substitution() {
        let value = "p@$(echo PWNED)`echo PWNED2`\"'\\";
        let (env, stdout) = run_github_script(&format_github("PASS", value, false));
        assert_eq!(env, format!("PASS={value}\n"));
        assert!(!stdout.contains("PWNED"));
    }

    #[cfg(unix)]
    #[test]
    fn github_multiline_value_cannot_inject_variables() {
        let value = "first\nNODE_OPTIONS=--require=/tmp/evil.js";
        let (env, _) = run_github_script(&format_github("CERT", value, false));
        let delim = github_delimiter(value);
        assert_eq!(env, format!("CERT<<{delim}\n{value}\n{delim}\n"));
    }

    #[cfg(unix)]
    #[test]
    fn gitlab_export_preserves_value_exactly() {
        let value = "a b $(id) `id` 'q' \"dq\" \\";
        let script = format!("{}printf '%s' \"$V\"", format_gitlab("V", value));
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(out.stdout).unwrap(), value);
    }

    #[test]
    fn generic_plain_value_is_unquoted() {
        assert_eq!(format_generic("HOST", "localhost"), "HOST=localhost\n");
        assert_eq!(
            format_generic("URL", "postgres://u@h:5432/db"),
            "URL=postgres://u@h:5432/db\n"
        );
        assert_eq!(format_generic("EMPTY", ""), "EMPTY=\n");
    }

    #[test]
    fn generic_special_values_are_quoted() {
        assert_eq!(format_generic("H", "abc # def"), "H='abc # def'\n");
        assert_eq!(format_generic("L", "$HOME"), "L='$HOME'\n");
        assert_eq!(format_generic("Q", "it's"), "Q=\"it's\"\n");
        assert_eq!(format_generic("M", "a\nb"), "M=\"a\\nb\"\n");
    }
}
