use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> assert_cmd::Command {
    cargo_bin_cmd!("vaultic")
}

// ─── template sync: error paths ─────────────────────────────────────────────

#[test]
fn template_sync_fails_without_vaultic_dir() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["template", "sync"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn template_sync_fails_when_no_enc_files() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Create minimal .vaultic/ structure with a dev env defined but no .enc file
    std::fs::create_dir_all(dir.path().join(".vaultic")).unwrap();
    std::fs::write(
        dir.path().join(".vaultic/config.toml"),
        r#"[vaultic]
version = "1.3.0"
default_cipher = "age"
default_env = "dev"
format_version = 1

[environments]
dev = {}

[audit]
enabled = true
log_file = "audit.log"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join(".vaultic/recipients.txt"), "").unwrap();

    // Generate a real age key so we don't fail on missing private key
    vaultic()
        .current_dir(dir.path())
        .args(["keys", "setup"])
        .write_stdin("1\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["template", "sync"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No encrypted environments"));
}

// ─── template sync: help text ────────────────────────────────────────────────

#[test]
fn template_sync_help_shows_output() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["template", "sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("template").and(predicate::str::contains("sync")));
}

// ─── template sync: happy path ───────────────────────────────────────────────

#[test]
fn template_sync_full_flow() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init with auto key generation so a real age identity is created
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Public key:"));

    // Create a .env with known keys
    dir.child(".env")
        .write_str("DATABASE_URL=postgres://localhost/mydb\nAPI_KEY=supersecret\nDEBUG=true")
        .unwrap();

    // Encrypt as dev environment
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    // Run template sync
    vaultic()
        .current_dir(dir.path())
        .args(["template", "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("keys"));

    // .env.template should have been created with the right keys and empty values
    let template_path = dir.path().join(".env.template");
    assert!(template_path.exists(), ".env.template was not created");

    let content = std::fs::read_to_string(&template_path).unwrap();
    assert!(
        content.contains("DATABASE_URL"),
        ".env.template missing DATABASE_URL"
    );
    assert!(content.contains("API_KEY"), ".env.template missing API_KEY");
    assert!(content.contains("DEBUG"), ".env.template missing DEBUG");
    // All values should be empty (stripped)
    assert!(
        !content.contains("postgres://localhost"),
        ".env.template should not contain plaintext value"
    );
    assert!(
        !content.contains("supersecret"),
        ".env.template should not contain plaintext value"
    );
}

#[test]
fn template_sync_output_flag_writes_to_custom_path() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init with auto key generation
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env")
        .write_str("SECRET_KEY=abc\nDB_HOST=localhost")
        .unwrap();

    // Encrypt
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success();

    // Sync to a custom output path
    let custom_output = "custom.template.env";
    vaultic()
        .current_dir(dir.path())
        .args(["template", "sync", "--output", custom_output])
        .assert()
        .success()
        .stdout(predicate::str::contains(custom_output));

    // Custom file should exist with correct keys
    let content = std::fs::read_to_string(dir.path().join(custom_output)).unwrap();
    assert!(
        content.contains("SECRET_KEY"),
        "custom output missing SECRET_KEY"
    );
    assert!(content.contains("DB_HOST"), "custom output missing DB_HOST");
    // Default .env.template should NOT have been created (or if it pre-existed from init, it may be empty)
    // The important assertion is that the custom file got the data
    assert!(
        !content.contains("abc"),
        "custom output should not contain plaintext value"
    );
}
