use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> Command {
    cargo_bin_cmd!("vaultic")
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
