use std::io::Write;
use std::path::Path;

use crate::core::errors::Result;

/// Write `data` to `path` atomically, readable only by the current user.
///
/// The content is written to a temporary file in the same directory
/// (created with mode 0600 on Unix), flushed to disk, and then renamed
/// over `path`. A crash never leaves a half-written file behind, and
/// plaintext secrets or private keys are never world-readable, even
/// for a moment.
pub fn write_private(path: &Path, data: &[u8]) -> Result<()> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    std::fs::create_dir_all(dir)?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(data)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

/// Create `path` (and its parents) as a directory accessible only by
/// the current user (mode 0700 on Unix). Existing directories are left
/// untouched.
pub fn create_private_dir(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }

    #[cfg(not(unix))]
    std::fs::create_dir_all(path)?;

    Ok(())
}

/// Return `true` if the file at `path` is readable by group or others.
///
/// Always `false` on non-Unix platforms, where permissions are ACL-based.
pub fn is_world_readable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.permissions().mode() & 0o077 != 0;
        }
    }

    #[cfg(not(unix))]
    let _ = path;

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_private_creates_file_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("secret.env");

        write_private(&path, b"KEY=value\n").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"KEY=value\n");
    }

    #[test]
    fn write_private_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.env");
        std::fs::write(&path, "OLD=1\n").unwrap();

        write_private(&path, b"NEW=2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "NEW=2\n");
    }

    #[cfg(unix)]
    #[test]
    fn write_private_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.env");
        std::fs::write(&path, "OLD=1\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(is_world_readable(&path));

        write_private(&path, b"NEW=2\n").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        assert!(!is_world_readable(&path));
    }

    #[cfg(unix)]
    #[test]
    fn create_private_dir_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("age");

        create_private_dir(&path).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }
}
