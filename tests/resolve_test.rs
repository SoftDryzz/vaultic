use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> Command {
    cargo_bin_cmd!("vaultic")
}

/// Helper: init project with key generation and create encrypted env files.
///
/// Sets up a project with base.env.enc and a named env.enc, ready for
/// resolve or env-diff testing.
fn setup_multi_env(
    dir: &assert_fs::TempDir,
    base_content: &str,
    env_name: &str,
    env_content: &str,
) {
    // Init with auto key generation
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Encrypt base environment
    dir.child(".env").write_str(base_content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "base"])
        .assert()
        .success();

    // Encrypt target environment
    std::fs::write(dir.path().join(".env"), env_content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", env_name])
        .assert()
        .success();

    // Remove .env so resolve has to generate it
    std::fs::remove_file(dir.path().join(".env")).unwrap();
}

#[test]
fn resolve_merges_base_and_dev() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_multi_env(
        &dir,
        "DB_HOST=localhost\nDB_PORT=5432",
        "dev",
        "DB_HOST=dev-db\nDEBUG=true",
    );

    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("base -> dev"))
        .stdout(predicate::str::contains("Resolved"))
        .stdout(predicate::str::contains("Written to .env"));

    // Verify the resolved .env has merged values
    let resolved = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(resolved.contains("DB_HOST=dev-db"), "overlay should win");
    assert!(resolved.contains("DB_PORT=5432"), "base value preserved");
    assert!(resolved.contains("DEBUG=true"), "new key from overlay");
}

#[test]
fn resolve_without_init_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "dev"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn resolve_unknown_env_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn resolve_base_only_works() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init with key
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Encrypt base
    dir.child(".env").write_str("SHARED_KEY=common").unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "base"])
        .assert()
        .success();

    std::fs::remove_file(dir.path().join(".env")).unwrap();

    // Resolve base (no inheritance)
    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "base"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 layer"));

    let resolved = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(resolved.contains("SHARED_KEY=common"));
}

#[test]
fn resolve_with_output_flag_writes_to_custom_path() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_multi_env(
        &dir,
        "DB_HOST=localhost\nDB_PORT=5432",
        "dev",
        "DB_HOST=dev-db\nDEBUG=true",
    );

    // Create target subdirectory
    std::fs::create_dir_all(dir.path().join("backend")).unwrap();

    // Resolve with --output pointing to subdirectory
    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "dev", "--output", "backend/.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Written to backend/.env"));

    // File should exist at custom path with merged content
    let content = std::fs::read_to_string(dir.path().join("backend/.env")).unwrap();
    assert!(content.contains("DB_HOST=dev-db"), "overlay should win");
    assert!(content.contains("DB_PORT=5432"), "base value preserved");

    // File should NOT exist at default .env path
    assert!(!dir.path().join(".env").exists());
}

#[test]
fn resolve_with_short_output_flag() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init with key
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Encrypt base
    dir.child(".env").write_str("APP_NAME=vaultic").unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "base"])
        .assert()
        .success();

    std::fs::remove_file(dir.path().join(".env")).unwrap();

    // Resolve with -o short flag
    vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "base", "-o", "resolved.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Written to resolved.env"));

    let content = std::fs::read_to_string(dir.path().join("resolved.env")).unwrap();
    assert!(content.contains("APP_NAME=vaultic"));
}

#[test]
fn diff_env_shows_differences() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Encrypt dev
    dir.child(".env")
        .write_str("DB_HOST=localhost\nDEBUG=true")
        .unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    // Encrypt prod (different values, missing DEBUG)
    std::fs::write(
        dir.path().join(".env"),
        "DB_HOST=rds.aws.com\nREDIS=prod-redis",
    )
    .unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "prod"])
        .assert()
        .success();

    // Diff between environments
    vaultic()
        .current_dir(dir.path())
        .args(["diff", "--env", "dev", "--env", "prod"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DB_HOST"))
        .stdout(predicate::str::contains("modified"));
}

#[test]
fn diff_env_identical_shows_no_differences() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    // Encrypt same content as both dev and staging
    dir.child(".env").write_str("KEY=same_value").unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "staging"])
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "--env", "dev", "--env", "staging"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences"));
}
