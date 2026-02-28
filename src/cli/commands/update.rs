use crate::adapters::updater::github_updater;
use crate::adapters::updater::verifier;
use crate::cli::output;
use crate::core::errors::Result;
use crate::core::models::update_info::current_version;

/// Execute the `vaultic update` command.
///
/// Checks for a newer release on GitHub, downloads the binary for the
/// current platform, verifies its SHA256 checksum and minisign signature,
/// and replaces the running binary.
pub fn execute() -> Result<()> {
    output::header("🔄 Vaultic — Update");

    // 1. Check for newer version
    let sp = output::spinner("Checking for updates...");
    let info = match github_updater::fetch_update_info()? {
        Some(info) => {
            output::finish_spinner(
                sp,
                &format!(
                    "New version available: {} → {}",
                    current_version(),
                    info.version
                ),
            );
            info
        }
        None => {
            output::finish_spinner(sp, &format!("Already up to date (v{})", current_version()));
            return Ok(());
        }
    };

    // 2. Download binary, checksums, and signature
    let sp = output::spinner(&format!("Downloading {}...", info.asset_name));
    let binary_data = github_updater::download_bytes(&info.asset_url)?;
    output::finish_spinner(sp, &format!("Downloaded {} bytes", binary_data.len()));

    let sp = output::spinner("Downloading verification files...");
    let checksums_data = github_updater::download_bytes(&info.checksums_url)?;
    let signature_data = github_updater::download_bytes(&info.signature_url)?;
    output::finish_spinner(sp, "Verification files downloaded");

    // 3. Verify signature of SHA256SUMS.txt
    let sp = output::spinner("Verifying cryptographic signature...");
    verifier::verify_signature(&checksums_data, &signature_data)?;
    output::finish_spinner(sp, "Signature valid (minisign Ed25519)");

    // 4. Verify SHA256 checksum of the binary
    let sp = output::spinner("Verifying SHA256 checksum...");
    let checksums_str = String::from_utf8_lossy(&checksums_data);
    verifier::verify_sha256(&binary_data, &info.asset_name, &checksums_str)?;
    output::finish_spinner(sp, "Checksum verified");

    // 5. Write to temp file and replace the running binary
    let sp = output::spinner("Installing update...");
    let tmp_path = std::env::temp_dir().join(&info.asset_name);
    std::fs::write(&tmp_path, &binary_data).map_err(|e| {
        crate::core::errors::VaulticError::UpdateFailed {
            reason: format!("Failed to write temp file: {e}"),
        }
    })?;
    self_replace::self_replace(&tmp_path).map_err(|e| {
        crate::core::errors::VaulticError::UpdateFailed {
            reason: format!("Failed to replace binary: {e}"),
        }
    })?;
    let _ = std::fs::remove_file(&tmp_path);
    output::finish_spinner(sp, &format!("Updated to v{}", info.version));

    output::success(&format!("Release notes: {}", info.release_url));
    output::success("Restart vaultic to use the new version.");

    Ok(())
}
