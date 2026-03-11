use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> assert_cmd::Command {
    cargo_bin_cmd!("vaultic")
}

/// Create a `.vaultic/config.toml` with the given validation rules.
fn setup_vaultic(dir: &assert_fs::TempDir, validation_rules: &str) {
    let config = format!(
        "[vaultic]\n\
         version = \"1.3.0\"\n\
         format_version = 1\n\
         default_cipher = \"age\"\n\
         default_env = \"dev\"\n\n\
         [environments]\n\
         dev = {{}}\n\n\
         [audit]\n\
         enabled = true\n\
         log_file = \"audit.log\"\n\n\
         [validation]\n\
         {validation_rules}"
    );
    dir.child(".vaultic/config.toml")
        .write_str(&config)
        .unwrap();
}

// ─── happy paths ────────────────────────────────────────────────────────────

#[test]
fn validate_all_pass() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(
        &dir,
        "PORT = { type = \"integer\" }\nDEBUG = { type = \"boolean\" }",
    );
    dir.child(".env")
        .write_str("PORT=8080\nDEBUG=true")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2 rules passed"));
}

#[test]
fn validate_exit_code_zero_on_all_pass() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "URL = { type = \"url\" }");
    dir.child(".env")
        .write_str("URL=https://example.com")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .success();
}

#[test]
fn validate_custom_file_flag() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "PORT = { type = \"integer\" }");
    dir.child("prod.env").write_str("PORT=443").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["validate", "-f", "prod.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1 rules passed"));
}

// ─── failure paths ──────────────────────────────────────────────────────────

#[test]
fn validate_fails_on_invalid_type() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "PORT = { type = \"integer\" }");
    dir.child(".env").write_str("PORT=not_a_number").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("PORT"))
        .stdout(predicate::str::contains("integer"));
}

#[test]
fn validate_fails_on_pattern_mismatch() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "STRIPE_KEY = { pattern = \"^sk_live_.*\" }");
    dir.child(".env")
        .write_str("STRIPE_KEY=sk_test_abc")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("STRIPE_KEY"))
        .stdout(predicate::str::contains("pattern"));
}

#[test]
fn validate_fails_on_required_missing() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "DB_URL = { required = true }");
    dir.child(".env").write_str("OTHER=value").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("DB_URL"))
        .stdout(predicate::str::contains("required"));
}

#[test]
fn validate_exit_code_nonzero_on_failure() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "URL = { type = \"url\" }");
    dir.child(".env").write_str("URL=not-a-url").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .failure();
}

// ─── edge cases ─────────────────────────────────────────────────────────────

#[test]
fn validate_warns_when_no_rules() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".vaultic/config.toml")
        .write_str(
            "[vaultic]\n\
             version = \"1.3.0\"\n\
             format_version = 1\n\
             default_cipher = \"age\"\n\
             default_env = \"dev\"\n\n\
             [environments]\n\
             dev = {}\n\n\
             [audit]\n\
             enabled = true\n\
             log_file = \"audit.log\"\n",
        )
        .unwrap();
    dir.child(".env").write_str("KEY=value").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("No [validation] rules"));
}

#[test]
fn validate_fails_without_env_file() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_vaultic(&dir, "KEY = { required = true }");

    vaultic()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains(".env"));
}
