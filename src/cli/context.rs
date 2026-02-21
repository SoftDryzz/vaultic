use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static VAULTIC_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the global vaultic directory path.
/// If `custom` is provided, uses that path; otherwise defaults to `.vaultic`.
pub fn init(custom: Option<&str>) {
    let dir = custom
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".vaultic"));
    let _ = VAULTIC_DIR.set(dir);
}

/// Get the current vaultic directory path.
pub fn vaultic_dir() -> &'static Path {
    VAULTIC_DIR
        .get()
        .map(|p| p.as_path())
        .unwrap_or(Path::new(".vaultic"))
}
