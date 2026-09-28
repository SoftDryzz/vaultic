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

    // Open the temp file with std's OpenOptions instead of tempfile's
    // defaults: on Windows tempfile sets FILE_ATTRIBUTE_TEMPORARY, which
    // would stick to `path` after the rename below.
    let mut tmp = tempfile::Builder::new().make_in(dir, |p| {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        opts.open(p)
    })?;
    tmp.write_all(data)?;
    tmp.as_file().sync_all()?;

    // Rename with std rather than `NamedTempFile::persist`: on Windows,
    // persist uses MoveFileExW, which fails with "Access is denied" when
    // another process has `path` open; std::fs::rename uses POSIX
    // semantics there. If the rename fails, dropping `tmp_path` removes
    // the temporary file.
    let mut tmp_path = tmp.into_temp_path();
    std::fs::rename(&tmp_path, path)?;
    tmp_path.disable_cleanup(true);
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

    #[test]
    fn write_private_replaces_file_open_by_another_handle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keys.txt");
        std::fs::write(&path, "OLD=1\n").unwrap();
        // Another process reading the key (e.g. a parallel `vaultic init`)
        let _reader = std::fs::File::open(&path).unwrap();

        write_private(&path, b"NEW=2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "NEW=2\n");
    }

    #[cfg(windows)]
    #[test]
    fn write_private_leaves_normal_file_attributes() {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keys.txt");

        write_private(&path, b"KEY=value\n").unwrap();

        let attrs = std::fs::metadata(&path).unwrap().file_attributes();
        assert_eq!(attrs & FILE_ATTRIBUTE_TEMPORARY, 0, "attrs=0x{attrs:x}");
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
