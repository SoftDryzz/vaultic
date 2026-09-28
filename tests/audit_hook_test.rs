use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;
use secrecy::ExposeSecret;

mod common;

/// Run vaultic with given args.
fn vaultic() -> Command {
    let mut cmd = cargo_bin_cmd!("vaultic");
    // Never hit the network from tests.
    cmd.env("VAULTIC_NO_UPDATE_CHECK", "1");
    // Never read or write the developer's real age key.
    cmd.env("VAULTIC_AGE_KEY_FILE", common::test_key_file());
    cmd
}

// ─── Audit / Log tests ───────────────────────────────────────────

#[test]
fn init_creates_audit_entry() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    let log_path = dir.path().join(".vaultic/audit.log");
    assert!(log_path.exists(), "audit.log should be created after init");

    let content = std::fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("\"action\":\"init\""));
}

#[test]
fn encrypt_creates_audit_entry() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env").write_str("KEY=value\n").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".vaultic/audit.log")).unwrap();
    assert!(content.contains("\"action\":\"encrypt\""));
}

#[test]
fn decrypt_audit_includes_destination_path() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Generate a local key to avoid race conditions in CI where
    // parallel tests overwrite the global ~/.config/age/keys.txt
    let identity = age::x25519::Identity::generate();
    let pubkey = identity.to_public().to_string();
    let key_path = dir.path().join("test_key.txt");
    let key_contents = format!(
        "# public key: {pubkey}\n{}\n",
        identity.to_string().expose_secret()
    );
    std::fs::write(&key_path, key_contents).unwrap();

    // Init without global key generation
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Add our local key as recipient
    vaultic()
        .current_dir(dir.path())
        .args(["keys", "add", &pubkey])
        .assert()
        .success();

    dir.child(".env").write_str("SECRET=audit_test\n").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    std::fs::remove_file(dir.path().join(".env")).unwrap();

    // Decrypt with custom output and explicit local key
    vaultic()
        .current_dir(dir.path())
        .args([
            "decrypt",
            "--env",
            "dev",
            "-o",
            "custom.env",
            "--key",
            key_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let log = std::fs::read_to_string(dir.path().join(".vaultic/audit.log")).unwrap();
    // Audit detail should mention the destination path
    assert!(
        log.contains("custom.env"),
        "audit log should include destination path"
    );
}

#[test]
fn log_shows_entries() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .arg("log")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}

#[test]
fn log_empty_no_entries() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Clear the audit log
    std::fs::write(dir.path().join(".vaultic/audit.log"), "").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("log")
        .assert()
        .success()
        .stdout(predicate::str::contains("No audit entries found"));
}

#[test]
fn log_filter_author_no_match() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["log", "--author", "nonexistent-user-xyz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No audit entries found"));
}

#[test]
fn log_last_limits_entries() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env").write_str("A=1\n").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    // Should have 2 entries (init + encrypt), show last 1
    vaultic()
        .current_dir(dir.path())
        .args(["log", "--last", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 entries"));
}

#[test]
fn log_without_init_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("log")
        .assert()
        .failure();
}

#[test]
fn log_invalid_since_date_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["log", "--since", "not-a-date"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid date format"));
}

// ─── Status tests ────────────────────────────────────────────────

#[test]
fn status_shows_project_info() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vaultic"))
        .stdout(predicate::str::contains("Cipher"))
        .stdout(predicate::str::contains("Recipients"));
}

#[test]
fn status_shows_env_files() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Should show environments as not encrypted
    vaultic()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("not encrypted"));
}

#[test]
fn status_without_init_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure();
}

// ─── Hook tests ──────────────────────────────────────────────────

#[test]
fn hook_install_and_uninstall() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Install hook
    vaultic()
        .current_dir(dir.path())
        .args(["hook", "install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pre-commit hook installed"));

    assert!(dir.path().join(".git/hooks/pre-commit").exists());

    // Uninstall hook
    vaultic()
        .current_dir(dir.path())
        .args(["hook", "uninstall"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pre-commit hook removed"));

    assert!(!dir.path().join(".git/hooks/pre-commit").exists());
}

/// Run a git command in `dir` with a throwaway identity.
fn git(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// Create a git repo in a temp dir with vaultic initialized and the hook installed.
fn repo_with_hook() -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    git(dir.path(), &["init", "-q"]);
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();
    vaultic()
        .current_dir(dir.path())
        .args(["hook", "install"])
        .assert()
        .success();
    dir
}

/// Commit with the hook enabled and return (success, stdout + stderr).
fn commit(dir: &std::path::Path, msg: &str) -> (bool, String) {
    let out = git(dir, &["commit", "-q", "-m", msg]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

#[test]
fn hook_blocks_env_in_non_ascii_directory() {
    let dir = repo_with_hook();

    // git C-quotes non-ASCII paths by default ("configuraci\303\263n/.env"),
    // which must not hide the file from the hook.
    dir.child("configuración/.env")
        .write_str("SECRET=1\n")
        .unwrap();
    git(dir.path(), &["add", "-f", "configuración/.env"]);

    let (ok, output) = commit(dir.path(), "leak");
    assert!(!ok, "non-ASCII nested .env should be blocked");
    assert!(output.contains("configuración/.env"), "output: {output}");
}

#[cfg(unix)]
#[test]
fn hook_blocks_symlink_replaced_by_env_file() {
    let dir = repo_with_hook();

    // Track .env as a symlink to the template (bypassing the hook)
    dir.child(".env.example").write_str("SECRET=\n").unwrap();
    std::os::unix::fs::symlink(".env.example", dir.path().join(".env")).unwrap();
    git(dir.path(), &["add", "-f", ".env.example", ".env"]);
    let out = git(
        dir.path(),
        &["commit", "-q", "--no-verify", "-m", "symlink"],
    );
    assert!(out.status.success());

    // Replacing the symlink with real secrets is a type change (T)
    std::fs::remove_file(dir.path().join(".env")).unwrap();
    dir.child(".env").write_str("SECRET=real\n").unwrap();
    git(dir.path(), &["add", "-f", ".env"]);

    let (ok, output) = commit(dir.path(), "leak");
    assert!(
        !ok,
        "symlink replaced by a real .env should be blocked: {output}"
    );
}

#[test]
fn hook_blocks_nested_env_and_allows_removal() {
    let dir = assert_fs::TempDir::new().unwrap();
    git(dir.path(), &["init"]);

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();
    vaultic()
        .current_dir(dir.path())
        .args(["hook", "install"])
        .assert()
        .success();

    // A nested .env in a directory with spaces must be blocked
    dir.child("my backend/.env")
        .write_str("SECRET=1\n")
        .unwrap();
    git(dir.path(), &["add", "-f", "my backend/.env"]);
    let out = git(dir.path(), &["commit", "-m", "leak"]);
    assert!(!out.status.success(), "nested .env should be blocked");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(combined.contains("my backend/.env"), "output: {combined}");

    // Templates are still allowed
    git(dir.path(), &["reset", "-q"]);
    dir.child("api/.env.example")
        .write_str("SECRET=\n")
        .unwrap();
    git(dir.path(), &["add", "api/.env.example"]);
    let out = git(dir.path(), &["commit", "-m", "template"]);
    assert!(out.status.success(), "templates should be allowed");

    // Removing a leaked .env (committed with --no-verify) must be allowed
    git(dir.path(), &["add", "-f", "my backend/.env"]);
    let out = git(dir.path(), &["commit", "--no-verify", "-m", "leak"]);
    assert!(out.status.success());
    git(dir.path(), &["rm", "--cached", "-q", "my backend/.env"]);
    let out = git(dir.path(), &["commit", "-m", "remove leak"]);
    assert!(
        out.status.success(),
        "removing a .env should not be blocked"
    );
}

#[test]
fn hook_install_without_git_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["hook", "install"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));
}

#[test]
fn hook_install_refuses_foreign_hook() {
    let dir = assert_fs::TempDir::new().unwrap();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Create a foreign pre-commit hook
    std::fs::create_dir_all(dir.path().join(".git/hooks")).unwrap();
    std::fs::write(
        dir.path().join(".git/hooks/pre-commit"),
        "#!/bin/sh\necho custom hook\n",
    )
    .unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["hook", "install"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed by Vaultic"));
}
