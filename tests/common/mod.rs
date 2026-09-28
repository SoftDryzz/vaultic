//! Helpers shared by the integration tests.

use std::path::PathBuf;

thread_local! {
    /// Per-test directory for the age identity. libtest runs each test on
    /// its own thread, so every test gets its own key, and the directory is
    /// removed when the test's thread ends.
    static KEY_DIR: tempfile::TempDir = tempfile::TempDir::new().unwrap();
}

/// Age identity file for the current test, passed to vaultic through
/// `VAULTIC_AGE_KEY_FILE` so tests never read or write the developer's
/// real key and parallel tests never share one.
pub fn test_key_file() -> PathBuf {
    KEY_DIR.with(|dir| dir.path().join("keys.txt"))
}
