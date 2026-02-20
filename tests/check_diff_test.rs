use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Run vaultic with given args.
fn vaultic() -> assert_cmd::Command {
    cargo_bin_cmd!("vaultic")
}

// ─── Check command ──────────────────────────────────────────────

#[test]
fn check_all_present() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost\nDB_PORT=5432\nAPI_KEY=secret")
        .unwrap();
    dir.child(".env.template")
        .write_str("DB_HOST=\nDB_PORT=\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("3/3 variables present"));
}

#[test]
fn check_missing_variables() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env").write_str("DB_HOST=localhost").unwrap();
    dir.child(".env.template")
        .write_str("DB_HOST=\nAPI_KEY=\nSECRET=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Missing variables (2)"))
        .stdout(predicate::str::contains("API_KEY"))
        .stdout(predicate::str::contains("SECRET"));
}

#[test]
fn check_extra_variables() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost\nOLD_VAR=legacy")
        .unwrap();
    dir.child(".env.template").write_str("DB_HOST=").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extra variables"))
        .stdout(predicate::str::contains("OLD_VAR"));
}

#[test]
fn check_empty_values() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("DB_HOST=localhost\nAPI_KEY=")
        .unwrap();
    dir.child(".env.template")
        .write_str("DB_HOST=\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty values"))
        .stdout(predicate::str::contains("API_KEY"));
}

#[test]
fn check_missing_env_file_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env.template").write_str("DB_HOST=").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(".env"));
}

#[test]
fn check_missing_template_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env").write_str("DB_HOST=localhost").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(".env.template"));
}

// ─── Diff command ───────────────────────────────────────────────

#[test]
fn diff_identical_files() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("a.env").write_str("KEY=value").unwrap();
    dir.child("b.env").write_str("KEY=value").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "a.env", "b.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn diff_shows_changes() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("dev.env")
        .write_str("DB=localhost\nDEBUG=true")
        .unwrap();
    dir.child("prod.env")
        .write_str("DB=rds.aws.com\nREDIS=redis.prod")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "dev.env", "prod.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DB"))
        .stdout(predicate::str::contains("DEBUG"))
        .stdout(predicate::str::contains("REDIS"));
}

#[test]
fn diff_summary_counts() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("a.env")
        .write_str("KEEP=same\nOLD=gone\nCHANGED=old")
        .unwrap();
    dir.child("b.env")
        .write_str("KEEP=same\nNEW=fresh\nCHANGED=new")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "a.env", "b.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 added"))
        .stdout(predicate::str::contains("1 removed"))
        .stdout(predicate::str::contains("1 modified"));
}

#[test]
fn diff_missing_file_fails() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("a.env").write_str("KEY=val").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "a.env", "nonexistent.env"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn diff_requires_two_files() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("a.env").write_str("KEY=val").unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "a.env"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires two files"));
}

#[test]
fn check_with_comments_in_files() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env")
        .write_str("# Database\nDB_HOST=localhost\n\n# API\nAPI_KEY=secret")
        .unwrap();
    dir.child(".env.template")
        .write_str("# Database\nDB_HOST=\n\n# API\nAPI_KEY=")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("2/2 variables present"));
}

#[test]
fn diff_with_quoted_values() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child("a.env")
        .write_str("SECRET=\"old secret\"")
        .unwrap();
    dir.child("b.env")
        .write_str("SECRET=\"new secret\"")
        .unwrap();

    vaultic()
        .current_dir(dir.path())
        .args(["diff", "a.env", "b.env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SECRET"))
        .stdout(predicate::str::contains("1 modified"));
}

#[test]
fn check_all_good_message() {
    let dir = assert_fs::TempDir::new().unwrap();

    dir.child(".env").write_str("KEY=value").unwrap();
    dir.child(".env.template").write_str("KEY=").unwrap();

    vaultic()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("all good"));
}
