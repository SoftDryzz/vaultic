use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args in a temp directory.
fn vaultic() -> Command {
    cargo_bin_cmd!("vaultic")
}

/// Generate a real age public key for testing.
fn generate_test_age_pubkey() -> String {
    let identity = age::x25519::Identity::generate();
    identity.to_public().to_string()
}

#[test]
fn init_creates_vaultic_directory() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Pass "n" to skip key generation (non-interactive)
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .vaultic/"))
        .stdout(predicate::str::contains("config.toml"));

    dir.child(".vaultic/config.toml")
        .assert(predicate::path::exists());
    dir.child(".vaultic/recipients.txt")
        .assert(predicate::path::exists());
    dir.child(".env.template").assert(predicate::path::exists());
}

#[test]
fn init_twice_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    // First init
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Second init should fail
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn encrypt_without_init_fails() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".env").write_str("KEY=value").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("encrypt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn decrypt_missing_file_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init without key gen
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .arg("decrypt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn encrypt_no_recipients_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init (may auto-detect system key)
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Force empty recipients to test the error path
    std::fs::write(dir.path().join(".vaultic/recipients.txt"), "").unwrap();

    dir.child(".env")
        .write_str("DB_URL=postgres://localhost")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("encrypt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No recipients"));
}

#[test]
fn keys_list_empty() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Force empty recipients to test empty list output
    std::fs::write(dir.path().join(".vaultic/recipients.txt"), "").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No recipients"));
}

#[test]
fn keys_add_and_list() {
    let dir = assert_fs::TempDir::new().unwrap();
    let pubkey = generate_test_age_pubkey();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Add a real age key
    vaultic()
        .current_dir(dir.path())
        .args(["keys", "add", &pubkey])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added recipient"));

    // List should show it
    vaultic()
        .current_dir(dir.path())
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&pubkey));
}

#[test]
fn keys_add_duplicate_fails() {
    let dir = assert_fs::TempDir::new().unwrap();
    let pubkey = generate_test_age_pubkey();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "add", &pubkey])
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "add", &pubkey])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn keys_remove() {
    let dir = assert_fs::TempDir::new().unwrap();
    let pubkey = generate_test_age_pubkey();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "add", &pubkey])
        .assert()
        .success();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "remove", &pubkey])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed recipient"));
}

#[test]
fn full_encrypt_decrypt_round_trip() {
    let dir = assert_fs::TempDir::new().unwrap();

    // Init with auto key generation
    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Public key:"));

    // Create a .env file
    let env_content = "DATABASE_URL=postgres://localhost/mydb\nAPI_KEY=supersecret\nDEBUG=true";
    dir.child(".env").write_str(env_content).unwrap();

    // Encrypt
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Encrypted with age"));

    // Verify encrypted file exists and is armored
    dir.child(".vaultic/dev.env.enc")
        .assert(predicate::path::exists());
    dir.child(".vaultic/dev.env.enc")
        .assert(predicate::str::contains("BEGIN AGE ENCRYPTED FILE"));

    // Delete the .env to prove decrypt works
    std::fs::remove_file(dir.path().join(".env")).unwrap();

    // Decrypt
    vaultic()
        .current_dir(dir.path())
        .args(["decrypt", "--env", "dev"])
        .assert()
        .success()
        .stdout(predicate::str::contains("3 variables"));

    // Verify decrypted content matches
    let decrypted = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(decrypted.contains("DATABASE_URL=postgres://localhost/mydb"));
    assert!(decrypted.contains("API_KEY=supersecret"));
    assert!(decrypted.contains("DEBUG=true"));
}

#[test]
fn encrypt_with_env_flag() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("y\n")
        .assert()
        .success();

    dir.child(".env").write_str("PROD_KEY=secret").unwrap();

    // Encrypt as prod
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", "prod"])
        .assert()
        .success();

    // Should create prod.env.enc
    dir.child(".vaultic/prod.env.enc")
        .assert(predicate::path::exists());
}

#[test]
fn unknown_cipher_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    dir.child(".env").write_str("KEY=val").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--cipher", "unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown cipher"));
}

#[test]
fn keys_add_with_label_shows_in_list() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Add key with a label comment in the recipients file
    let recipients_path = dir.path().join(".vaultic/recipients.txt");
    std::fs::write(&recipients_path, "age1labeltest # team-lead\n").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("age1labeltest"))
        .stdout(predicate::str::contains("team-lead"));
}
