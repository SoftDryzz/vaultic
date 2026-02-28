use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> assert_cmd::Command {
    cargo_bin_cmd!("vaultic")
}

// ─── Template auto-discovery ─────────────────────────────────────

#[test]
fn check_discovers_env_template() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost\nAPI_KEY=secret")
        .unwrap();
    dir.child(".env.template")
        .write_str("DB_HOST=\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2 variables present"));
}

#[test]
fn check_discovers_env_example_as_fallback() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost\nAPI_KEY=secret")
        .unwrap();
    // No .env.template — only .env.example
    dir.child(".env.example")
        .write_str("DB_HOST=\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2 variables present"));
}

#[test]
fn check_discovers_env_sample_as_fallback() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost")
        .unwrap();
    // No .env.template or .env.example — only .env.sample
    dir.child(".env.sample")
        .write_str("DB_HOST=\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Missing variables (1)"));
}

#[test]
fn check_prefers_env_template_over_env_example() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost")
        .unwrap();
    // .env.template has 1 key, .env.example has 2 keys
    dir.child(".env.template")
        .write_str("DB_HOST=")
        .unwrap();
    dir.child(".env.example")
        .write_str("DB_HOST=\nAPI_KEY=")
        .unwrap();

    // Should use .env.template (1 key = 1/1) not .env.example (1/2)
    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1 variables present"));
}

#[test]
fn check_no_template_found_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost")
        .unwrap();
    // No template file at all

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("template"));
}

// ─── Update command ─────────────────────────────────────────────

#[test]
fn update_help_shows_description() {
    vaultic()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("latest Vaultic release"));
}

// ─── Format version ─────────────────────────────────────────────

#[test]
fn init_includes_format_version() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .arg("--cipher")
        .arg("age")
        .assert()
        .success();

    let config = std::fs::read_to_string(dir.path().join(".vaultic/config.toml")).unwrap();
    assert!(
        config.contains("format_version"),
        "config.toml should contain format_version"
    );
}
