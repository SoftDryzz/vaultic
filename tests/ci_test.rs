use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn vaultic() -> Command {
    cargo_bin_cmd!("vaultic")
}

fn setup_env(dir: &assert_fs::TempDir, env_name: &str, content: &str) {
    if !dir.path().join(".vaultic").exists() {
        vaultic()
            .current_dir(dir.path())
            .arg("init")
            .write_stdin("y\n")
            .assert()
            .success();
    }

    dir.child(".env").write_str(content).unwrap();
    vaultic()
        .current_dir(dir.path())
        .args(["encrypt", "--env", env_name])
        .assert()
        .success();
    std::fs::remove_file(dir.path().join(".env")).unwrap();
}

#[test]
fn ci_export_generic_format() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "DB_HOST=localhost\nAPI_KEY=secret123");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev", "--format", "generic"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DB_HOST=localhost"));
    assert!(stdout.contains("API_KEY=secret123"));
    assert!(output.status.success());
}

#[test]
fn ci_export_github_format() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "DB_HOST=localhost\nAPI_KEY=secret123");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev", "--format", "github"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("echo \"DB_HOST=localhost\" >> \"$GITHUB_ENV\""));
    assert!(stdout.contains("echo \"API_KEY=secret123\" >> \"$GITHUB_ENV\""));
}

#[test]
fn ci_export_github_with_mask() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "API_KEY=secret123");

    let output = vaultic()
        .current_dir(dir.path())
        .args([
            "ci", "export", "--env", "dev", "--format", "github", "--mask",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("echo \"::add-mask::secret123\""));
    assert!(stdout.contains("echo \"API_KEY=secret123\" >> \"$GITHUB_ENV\""));
}

#[test]
fn ci_export_gitlab_format() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "DB_HOST=localhost\nAPI_KEY=secret123");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev", "--format", "gitlab"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("export DB_HOST=\"localhost\""));
    assert!(stdout.contains("export API_KEY=\"secret123\""));
}

#[test]
fn ci_export_default_format_is_generic() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "KEY=value");

    let output = vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("KEY=value"));
    assert!(!stdout.contains("echo"));
    assert!(!stdout.contains("export"));
}

#[test]
fn ci_export_without_init_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn ci_export_invalid_format_fails() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "KEY=value");

    vaultic()
        .current_dir(dir.path())
        .args(["ci", "export", "--env", "dev", "--format", "jenkins"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid CI format"));
}

#[test]
fn ci_export_mask_without_github_fails() {
    let dir = assert_fs::TempDir::new().unwrap();
    setup_env(&dir, "dev", "KEY=value");

    vaultic()
        .current_dir(dir.path())
        .args([
            "ci", "export", "--env", "dev", "--format", "gitlab", "--mask",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--mask is only supported with --format github",
        ));
}
