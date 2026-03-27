use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args in a temp directory.
fn vaultic() -> Command {
    cargo_bin_cmd!("vaultic")
}

/// Helper: init project with key generation, encrypt a .env as the given env.
fn setup_encrypted_env(dir: &assert_fs::TempDir, env_name: &str, content: &str) {
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env").write_str(content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", env_name])
        .assert()
        .success();

    std::fs::remove_file(dir.path().join(".env")).unwrap();
}

/// Helper: init + encrypt base and a named env for resolve tests.
fn setup_multi_env(
    dir: &assert_fs::TempDir,
    base_content: &str,
    env_name: &str,
    env_content: &str,
) {
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env").write_str(base_content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "base"])
        .assert()
        .success();

    std::fs::write(dir.path().join(".env"), env_content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", env_name])
        .assert()
        .success();

    std::fs::remove_file(dir.path().join(".env")).unwrap();
}

#[test]
fn decrypt_stdout_prints_env_to_stdout() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_encrypted_env(&dir, "dev", "DB_HOST=localhost\nAPI_KEY=secret123");

    vaultic()
        .current_dir(dir.path())
        .args(["decrypt", "--env", "dev", "--stdout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DB_HOST=localhost"))
        .stdout(predicate::str::contains("API_KEY=secret123"));

    // .env should NOT be written to disk
    assert!(!dir.path().join(".env").exists());
}

#[test]
fn decrypt_stdout_suppresses_ui_output() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_encrypted_env(&dir, "dev", "KEY=value");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["decrypt", "--env", "dev", "--stdout"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("KEY=value"));
    assert!(!stdout.contains("Decrypted"));
    assert!(!stdout.contains("Generated"));
}

#[test]
fn resolve_stdout_prints_merged_env() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_multi_env(
        &dir,
        "DB_HOST=localhost\nDB_PORT=5432",
        "dev",
        "DB_HOST=dev-db\nDEBUG=true",
    );

    let output = vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "dev", "--stdout"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DB_HOST=dev-db"), "overlay wins");
    assert!(stdout.contains("DB_PORT=5432"), "base preserved");
    assert!(stdout.contains("DEBUG=true"), "new key from overlay");
    assert!(!dir.path().join(".env").exists());
}

#[test]
fn resolve_stdout_suppresses_ui_output() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_multi_env(&dir, "BASE_KEY=1", "dev", "DEV_KEY=2");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["resolve", "--env", "dev", "--stdout"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("BASE_KEY=1"));
    assert!(stdout.contains("DEV_KEY=2"));
    assert!(!stdout.contains("Resolving"));
    assert!(!stdout.contains("Written to"));
    assert!(!stdout.contains("Inheritance chain"));
}

#[test]
fn stdout_and_output_are_mutually_exclusive() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_encrypted_env(&dir, "dev", "K=V");

    vaultic()
        .current_dir(dir.path())
        .args(["decrypt", "--env", "dev", "--stdout", "--output", "out.env"])
        .assert()
        .failure();
}

#[test]
fn resolve_stdout_and_output_are_mutually_exclusive() {
    let dir = assert_fs::TempDir::new().unwrap();

    setup_multi_env(&dir, "A=1", "dev", "B=2");

    vaultic()
        .current_dir(dir.path())
        .args([
            "resolve", "--env", "dev", "--stdout", "--output", "out.env",
        ])
        .assert()
        .failure();
}
