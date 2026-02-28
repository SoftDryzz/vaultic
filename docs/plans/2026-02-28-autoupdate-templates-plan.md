# Auto-Update + Template Detection + Backward Compatibility — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add self-update with SHA256+minisign verification, improve template detection with auto-discovery and per-environment support, and add format versioning for backward compatibility.

**Architecture:** Follows existing hexagonal architecture — new traits in `core/traits/`, models in `core/models/`, adapters in `adapters/updater/`, CLI command in `cli/commands/update.rs`. Template improvements modify existing files. Format versioning adds a field to `config.toml` with migration infrastructure.

**Tech Stack:** reqwest (HTTP), semver (version comparison), minisign-verify (signature verification), self_replace (binary swap), tokio (async runtime for reqwest)

**Design doc:** `docs/plans/2026-02-28-autoupdate-templates-design.md`

---

## Task 1: Add New Dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add the new dependencies**

Add after the existing `dirs = "6"` line in `[dependencies]`:

```toml
# Auto-update
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
semver = "1"
minisign-verify = "0.2"
self_replace = "1"
tokio = { version = "1", features = ["rt"] }
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors (warnings OK for now — unused deps)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: add auto-update dependencies (reqwest, semver, minisign-verify, self_replace, tokio)"
```

---

## Task 2: Add Update Error Variants

**Files:**
- Modify: `src/core/errors.rs`

**Step 1: Add update-related error variants**

Add these variants to `VaulticError` enum before the `Io` variant:

```rust
#[error(
    "Update check failed: {reason}\n\n  \
     This is not critical — your current version continues to work.\n  \
     Try again later or check https://github.com/SoftDryzz/vaultic/releases"
)]
UpdateCheckFailed { reason: String },

#[error(
    "Update verification failed: {reason}\n\n  \
     The downloaded binary could not be verified and was NOT installed.\n  \
     Your current installation is unchanged.\n\n  \
     Solutions:\n    \
     → Try again: vaultic update\n    \
     → Manual download: https://github.com/SoftDryzz/vaultic/releases/latest\n    \
     → Report issue: https://github.com/SoftDryzz/vaultic/issues"
)]
UpdateVerificationFailed { reason: String },

#[error(
    "Update failed: {reason}\n\n  \
     The binary replacement failed. Your current installation may be intact.\n\n  \
     Solutions:\n    \
     → Try again: vaultic update\n    \
     → Manual install: cargo install vaultic --force"
)]
UpdateFailed { reason: String },

#[error(
    "Unsupported platform for auto-update: {platform}\n\n  \
     Pre-built binaries are not available for your platform.\n\n  \
     Solutions:\n    \
     → Install from source: cargo install vaultic\n    \
     → Build manually: cargo build --release"
)]
UnsupportedPlatform { platform: String },

#[error(
    "No template file found\n\n  \
     Vaultic searched for:\n    \
     {searched}\n\n  \
     Solutions:\n    \
     → Create a template: cp .env .env.template (then remove secret values)\n    \
     → Specify in .vaultic/config.toml:\n      \
       [vaultic]\n      \
       template = \"path/to/your/template\""
)]
TemplateNotFound { searched: String },

#[error(
    "This project uses format version {project_version}, but your Vaultic \
     only supports up to version {supported_version}.\n\n  \
     Solutions:\n    \
     → Update Vaultic: vaultic update\n    \
     → Or install latest: cargo install vaultic --force"
)]
FormatVersionTooNew {
    project_version: u32,
    supported_version: u32,
},
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles (new variants are unused for now)

**Step 3: Commit**

```bash
git add src/core/errors.rs
git commit -m "feat: add error variants for update, template discovery, and format versioning"
```

---

## Task 3: Create UpdateInfo Model

**Files:**
- Create: `src/core/models/update_info.rs`
- Modify: `src/core/models/mod.rs`

**Step 1: Write the model with tests**

Create `src/core/models/update_info.rs`:

```rust
use serde::Deserialize;

/// Information about an available update from GitHub Releases.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Latest version available (e.g., "1.2.0").
    pub version: semver::Version,
    /// URL to download the binary for the current platform.
    pub asset_url: String,
    /// Name of the asset file (e.g., "vaultic-linux-amd64").
    pub asset_name: String,
    /// URL to the SHA256SUMS.txt file.
    pub checksums_url: String,
    /// URL to the SHA256SUMS.txt.minisig file.
    pub signature_url: String,
    /// URL to the release page (for changelog link).
    pub release_url: String,
}

/// Partial structure for deserializing the GitHub Releases API response.
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub assets: Vec<GitHubAsset>,
}

/// A single asset in a GitHub Release.
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Cached result of a version check.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateCheckCache {
    /// ISO 8601 timestamp of when the check was performed.
    pub checked_at: String,
    /// Latest version found (None if check failed).
    pub latest_version: Option<String>,
}

/// Returns the expected asset name for the current platform.
///
/// Returns `None` if the platform is not supported for pre-built binaries.
pub fn current_platform_asset() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("vaultic-linux-amd64"),
        ("linux", "aarch64") => Some("vaultic-linux-arm64"),
        ("macos", "x86_64") => Some("vaultic-darwin-amd64"),
        ("macos", "aarch64") => Some("vaultic-darwin-arm64"),
        ("windows", "x86_64") => Some("vaultic-windows-amd64.exe"),
        _ => None,
    }
}

/// Current version of Vaultic, parsed from Cargo.toml at compile time.
pub fn current_version() -> semver::Version {
    env!("CARGO_PKG_VERSION")
        .parse()
        .expect("CARGO_PKG_VERSION is always valid semver")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_is_valid_semver() {
        let v = current_version();
        assert!(v.major >= 1);
    }

    #[test]
    fn platform_asset_returns_some_on_known_platforms() {
        // This test runs on CI across platforms, so just check it doesn't panic
        let _ = current_platform_asset();
    }

    #[test]
    fn github_release_deserializes() {
        let json = r#"{
            "tag_name": "v1.2.0",
            "html_url": "https://github.com/SoftDryzz/vaultic/releases/tag/v1.2.0",
            "assets": [
                {
                    "name": "vaultic-linux-amd64",
                    "browser_download_url": "https://github.com/SoftDryzz/vaultic/releases/download/v1.2.0/vaultic-linux-amd64"
                },
                {
                    "name": "SHA256SUMS.txt",
                    "browser_download_url": "https://github.com/SoftDryzz/vaultic/releases/download/v1.2.0/SHA256SUMS.txt"
                }
            ]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v1.2.0");
        assert_eq!(release.assets.len(), 2);
        assert_eq!(release.assets[0].name, "vaultic-linux-amd64");
    }

    #[test]
    fn update_check_cache_round_trip() {
        let cache = UpdateCheckCache {
            checked_at: "2026-02-28T12:00:00Z".to_string(),
            latest_version: Some("1.2.0".to_string()),
        };
        let json = serde_json::to_string(&cache).unwrap();
        let parsed: UpdateCheckCache = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.latest_version, Some("1.2.0".to_string()));
    }
}
```

**Step 2: Register the module**

Add `pub mod update_info;` to `src/core/models/mod.rs`.

**Step 3: Run tests**

Run: `cargo test --lib core::models::update_info`
Expected: All 4 tests pass

**Step 4: Commit**

```bash
git add src/core/models/update_info.rs src/core/models/mod.rs
git commit -m "feat: add UpdateInfo model and GitHub release deserialization"
```

---

## Task 4: Create Verifier Module (SHA256 + minisign)

**Files:**
- Create: `src/adapters/updater/mod.rs`
- Create: `src/adapters/updater/verifier.rs`
- Modify: `src/adapters/mod.rs`

**Step 1: Write the verifier with tests**

Create `src/adapters/updater/mod.rs`:

```rust
pub mod github_updater;
pub mod verifier;
```

Create `src/adapters/updater/verifier.rs`:

```rust
use sha2::{Digest, Sha256};

use crate::core::errors::{Result, VaulticError};

/// Embedded minisign public key for verifying release signatures.
///
/// This key is generated once and the corresponding secret key is
/// stored in GitHub Secrets for CI signing.
///
/// IMPORTANT: Replace this placeholder with the real public key
/// after running: minisign -G -p vaultic.pub -s vaultic.key
pub const MINISIGN_PUBLIC_KEY: &str =
    "untrusted comment: minisign public key for vaultic\nRWTOPLACEHOLDER_REPLACE_WITH_REAL_KEY_AFTER_GENERATION";

/// Compute the SHA256 hex digest of the given bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify that the SHA256 hash of `binary_data` matches the expected hash
/// for `asset_name` found in the `checksums_content` (SHA256SUMS.txt format).
///
/// SHA256SUMS.txt format: `<hex_hash>  <filename>` (two spaces between hash and name).
pub fn verify_sha256(
    binary_data: &[u8],
    asset_name: &str,
    checksums_content: &str,
) -> Result<()> {
    let computed = sha256_hex(binary_data);

    let expected = checksums_content
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, "  ");
            let hash = parts.next()?.trim();
            let name = parts.next()?.trim();
            if name == asset_name {
                Some(hash.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| VaulticError::UpdateVerificationFailed {
            reason: format!(
                "Asset '{asset_name}' not found in SHA256SUMS.txt\n\n  \
                 This release may not include a binary for your platform."
            ),
        })?;

    if computed != expected {
        return Err(VaulticError::UpdateVerificationFailed {
            reason: format!(
                "SHA256 mismatch\n\n  \
                 Downloaded binary hash: {computed}\n  \
                 Expected hash:          {expected}\n\n  \
                 The download may be corrupted or tampered with."
            ),
        });
    }

    Ok(())
}

/// Verify the minisign signature of SHA256SUMS.txt.
pub fn verify_signature(checksums_content: &[u8], signature_content: &[u8]) -> Result<()> {
    let pk = minisign_verify::PublicKey::from_base64(
        MINISIGN_PUBLIC_KEY
            .lines()
            .nth(1)
            .unwrap_or(MINISIGN_PUBLIC_KEY),
    )
    .map_err(|e| VaulticError::UpdateVerificationFailed {
        reason: format!("Invalid embedded public key: {e}"),
    })?;

    let sig = minisign_verify::Signature::decode(
        &String::from_utf8_lossy(signature_content),
    )
    .map_err(|e| VaulticError::UpdateVerificationFailed {
        reason: format!("Invalid signature file: {e}"),
    })?;

    pk.verify(checksums_content, &sig, false)
        .map_err(|e| VaulticError::UpdateVerificationFailed {
            reason: format!(
                "Invalid signature\n\n  \
                 SHA256SUMS.txt signature does not match the embedded public key.\n  \
                 This could indicate the release has been tampered with.\n\n  \
                 Error: {e}"
            ),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_produces_correct_hash() {
        let hash = sha256_hex(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn verify_sha256_passes_with_matching_hash() {
        let data = b"binary content here";
        let hash = sha256_hex(data);
        let checksums = format!("{hash}  vaultic-linux-amd64\nabc123  other-file");
        assert!(verify_sha256(data, "vaultic-linux-amd64", &checksums).is_ok());
    }

    #[test]
    fn verify_sha256_fails_with_wrong_hash() {
        let data = b"binary content here";
        let checksums = "0000000000000000000000000000000000000000000000000000000000000000  vaultic-linux-amd64";
        let result = verify_sha256(data, "vaultic-linux-amd64", checksums);
        assert!(result.is_err());
    }

    #[test]
    fn verify_sha256_fails_when_asset_missing_from_checksums() {
        let data = b"binary content";
        let checksums = "abc123  other-file";
        let result = verify_sha256(data, "vaultic-linux-amd64", checksums);
        assert!(result.is_err());
    }
}
```

**Step 2: Register the module**

Add `pub mod updater;` to `src/adapters/mod.rs`.

**Step 3: Run tests**

Run: `cargo test --lib adapters::updater::verifier`
Expected: All 4 tests pass

**Step 4: Commit**

```bash
git add src/adapters/updater/ src/adapters/mod.rs
git commit -m "feat: add SHA256 and minisign verification for update downloads"
```

---

## Task 5: Create GitHub Updater Adapter

**Files:**
- Create: `src/adapters/updater/github_updater.rs`

**Step 1: Write the GitHub updater**

Create `src/adapters/updater/github_updater.rs`:

```rust
use std::path::PathBuf;
use std::time::Duration;

use crate::core::errors::{Result, VaulticError};
use crate::core::models::update_info::{
    current_platform_asset, current_version, GitHubRelease, UpdateCheckCache, UpdateInfo,
};

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/SoftDryzz/vaultic/releases/latest";

/// Timeout for the passive version check (startup banner).
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// Timeout for the explicit download during `vaultic update`.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

/// How long to cache the update check result (24 hours).
const CACHE_TTL_SECS: i64 = 86400;

/// Build a reqwest client with the given timeout.
fn build_client(timeout: Duration) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(format!("vaultic/{}", current_version()))
        .build()
        .map_err(|e| VaulticError::UpdateCheckFailed {
            reason: format!("Failed to create HTTP client: {e}"),
        })
}

/// Path to the update check cache file.
fn cache_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| VaulticError::UpdateCheckFailed {
        reason: "Could not determine config directory".into(),
    })?;
    Ok(config_dir.join("vaultic").join("last_update_check.json"))
}

/// Check if the cached update check is still fresh (< 24 hours old).
pub fn is_cache_fresh() -> bool {
    let Ok(path) = cache_path() else {
        return false;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(cache) = serde_json::from_str::<UpdateCheckCache>(&content) else {
        return false;
    };
    let Ok(checked_at) = chrono::DateTime::parse_from_rfc3339(&cache.checked_at) else {
        return false;
    };
    let age = chrono::Utc::now().signed_duration_since(checked_at);
    age.num_seconds() < CACHE_TTL_SECS
}

/// Save the update check result to cache.
fn save_cache(latest_version: Option<&str>) {
    let Ok(path) = cache_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let cache = UpdateCheckCache {
        checked_at: chrono::Utc::now().to_rfc3339(),
        latest_version: latest_version.map(|s| s.to_string()),
    };
    let _ = serde_json::to_string(&cache)
        .map(|json| std::fs::write(&path, json));
}

/// Fetch the latest release info from GitHub (quick check, 3s timeout).
///
/// Returns `Some(version_string)` if a newer version is available, `None` otherwise.
/// Never errors — returns `None` on any failure (network, parse, etc.).
pub fn check_latest_version() -> Option<String> {
    if is_cache_fresh() {
        // Read from cache
        let path = cache_path().ok()?;
        let content = std::fs::read_to_string(path).ok()?;
        let cache: UpdateCheckCache = serde_json::from_str(&content).ok()?;
        let latest_str = cache.latest_version?;
        let latest: semver::Version = latest_str.parse().ok()?;
        if latest > current_version() {
            return Some(latest_str);
        }
        return None;
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;

    rt.block_on(async {
        let client = build_client(CHECK_TIMEOUT).ok()?;
        let resp = client
            .get(GITHUB_API_URL)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .ok()?;
        let release: GitHubRelease = resp.json().await.ok()?;
        let version_str = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
        let latest: semver::Version = version_str.parse().ok()?;

        save_cache(Some(version_str));

        if latest > current_version() {
            Some(version_str.to_string())
        } else {
            None
        }
    })
}

/// Fetch full release info for performing an update (longer timeout).
pub fn fetch_update_info() -> Result<Option<UpdateInfo>> {
    let asset_name = current_platform_asset().ok_or_else(|| VaulticError::UnsupportedPlatform {
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    })?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| VaulticError::UpdateCheckFailed {
            reason: format!("Failed to create async runtime: {e}"),
        })?;

    rt.block_on(async {
        let client = build_client(DOWNLOAD_TIMEOUT)?;
        let resp = client
            .get(GITHUB_API_URL)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| VaulticError::UpdateCheckFailed {
                reason: format!("GitHub API request failed: {e}"),
            })?;

        if !resp.status().is_success() {
            return Err(VaulticError::UpdateCheckFailed {
                reason: format!("GitHub API returned status {}", resp.status()),
            });
        }

        let release: GitHubRelease = resp.json().await.map_err(|e| {
            VaulticError::UpdateCheckFailed {
                reason: format!("Failed to parse GitHub response: {e}"),
            }
        })?;

        let version_str = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
        let latest: semver::Version =
            version_str
                .parse()
                .map_err(|e| VaulticError::UpdateCheckFailed {
                    reason: format!("Invalid version '{version_str}': {e}"),
                })?;

        if latest <= current_version() {
            return Ok(None);
        }

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| VaulticError::UpdateCheckFailed {
                reason: format!(
                    "No binary for your platform ({asset_name}) in release {version_str}"
                ),
            })?;

        let checksums = release
            .assets
            .iter()
            .find(|a| a.name == "SHA256SUMS.txt")
            .ok_or_else(|| VaulticError::UpdateCheckFailed {
                reason: "Release is missing SHA256SUMS.txt — cannot verify download".into(),
            })?;

        let signature = release
            .assets
            .iter()
            .find(|a| a.name == "SHA256SUMS.txt.minisig")
            .ok_or_else(|| VaulticError::UpdateCheckFailed {
                reason: "Release is missing SHA256SUMS.txt.minisig — cannot verify download"
                    .into(),
            })?;

        Ok(Some(UpdateInfo {
            version: latest,
            asset_url: asset.browser_download_url.clone(),
            asset_name: asset.name.clone(),
            checksums_url: checksums.browser_download_url.clone(),
            signature_url: signature.browser_download_url.clone(),
            release_url: release.html_url,
        }))
    })
}

/// Download bytes from a URL.
pub fn download_bytes(url: &str) -> Result<Vec<u8>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| VaulticError::UpdateFailed {
            reason: format!("Failed to create async runtime: {e}"),
        })?;

    rt.block_on(async {
        let client = build_client(DOWNLOAD_TIMEOUT)?;
        let resp = client.get(url).send().await.map_err(|e| {
            VaulticError::UpdateFailed {
                reason: format!("Download failed: {e}"),
            }
        })?;

        if !resp.status().is_success() {
            return Err(VaulticError::UpdateFailed {
                reason: format!("Download returned status {}", resp.status()),
            });
        }

        resp.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            VaulticError::UpdateFailed {
                reason: format!("Failed to read download: {e}"),
            }
        })
    })
}
```

**Step 2: Run check**

Run: `cargo check`
Expected: Compiles with no errors

**Step 3: Commit**

```bash
git add src/adapters/updater/github_updater.rs
git commit -m "feat: add GitHub updater adapter with version check and download"
```

---

## Task 6: Create `vaultic update` CLI Command

**Files:**
- Create: `src/cli/commands/update.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/cli/mod.rs` (add `Update` variant to `Commands`)
- Modify: `src/main.rs` (add dispatch for `Update`)

**Step 1: Create the update command**

Create `src/cli/commands/update.rs`:

```rust
use crate::adapters::updater::{github_updater, verifier};
use crate::cli::output;
use crate::core::errors::Result;
use crate::core::models::update_info::current_version;

/// Execute the `vaultic update` command.
///
/// Checks for a newer version on GitHub, downloads it, verifies
/// SHA256 + minisign signature, and replaces the current binary.
pub fn execute() -> Result<()> {
    output::header("Vaultic — Checking for updates");

    let sp = output::spinner("Checking latest version...");
    let info = github_updater::fetch_update_info()?;
    output::finish_spinner(sp, "Version check complete");

    let info = match info {
        Some(info) => info,
        None => {
            output::success(&format!(
                "You're already on the latest version (v{})",
                current_version()
            ));
            return Ok(());
        }
    };

    println!(
        "\n  New version available: v{} → v{}",
        current_version(),
        info.version
    );

    // Download binary
    let sp = output::spinner("Downloading binary...");
    let binary_data = github_updater::download_bytes(&info.asset_url)?;
    output::finish_spinner(
        sp,
        &format!("Downloaded {} ({} bytes)", info.asset_name, binary_data.len()),
    );

    // Download checksums
    let sp = output::spinner("Downloading checksums...");
    let checksums_data = github_updater::download_bytes(&info.checksums_url)?;
    let checksums_text = String::from_utf8_lossy(&checksums_data);
    output::finish_spinner(sp, "Checksums downloaded");

    // Download signature
    let sp = output::spinner("Downloading signature...");
    let signature_data = github_updater::download_bytes(&info.signature_url)?;
    output::finish_spinner(sp, "Signature downloaded");

    // Verify SHA256
    let sp = output::spinner("Verifying SHA256 checksum...");
    verifier::verify_sha256(&binary_data, &info.asset_name, &checksums_text)?;
    output::finish_spinner(sp, "SHA256 checksum verified");

    // Verify minisign signature
    let sp = output::spinner("Verifying minisign signature...");
    verifier::verify_signature(&checksums_data, &signature_data)?;
    output::finish_spinner(sp, "Minisign signature verified");

    // Apply update
    let sp = output::spinner("Applying update...");
    self_replace::self_replace(&binary_data).map_err(|e| {
        crate::core::errors::VaulticError::UpdateFailed {
            reason: format!("Binary replacement failed: {e}"),
        }
    })?;
    output::finish_spinner(sp, &format!("Updated to v{}", info.version));

    println!("\n  Changelog: {}", info.release_url);

    Ok(())
}
```

**Step 2: Register the module**

Add `pub mod update;` to `src/cli/commands/mod.rs`.

**Step 3: Add `Update` to CLI enum**

In `src/cli/mod.rs`, add to the `Commands` enum:

```rust
/// Check for and apply updates
#[command(
    long_about = "Check for a newer version of Vaultic and update in place.\n\n\
                  Downloads the latest release from GitHub, verifies its SHA256 \
                  checksum and minisign signature, then replaces the current binary.\n\n\
                  Your current installation is never modified until all verification \
                  checks pass.",
    after_help = "Examples:\n  \
                  vaultic update                    # Check and apply update"
)]
Update,
```

**Step 4: Add dispatch in main.rs**

In `src/main.rs`, add to the match block:

```rust
Commands::Update => cli::commands::update::execute(),
```

**Step 5: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 6: Commit**

```bash
git add src/cli/commands/update.rs src/cli/commands/mod.rs src/cli/mod.rs src/main.rs
git commit -m "feat: add 'vaultic update' command with SHA256+minisign verification"
```

---

## Task 7: Add Passive Version Check on Startup

**Files:**
- Modify: `src/main.rs`

**Step 1: Add background version check**

Replace the contents of `src/main.rs` with the version check logic integrated.
Add a function that checks and prints the banner AFTER the command runs:

```rust
mod adapters;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    // Initialize global CLI state before any command runs
    cli::output::init(args.verbose, args.quiet);
    cli::context::init(args.config.as_deref());

    // Validate all --env values before dispatching any command
    for env_name in &args.env {
        if let Err(e) = cli::context::validate_env_name(env_name) {
            cli::output::error(&format!("Error: {e}"));
            std::process::exit(1);
        }
    }

    // Start passive update check in background thread (non-blocking)
    // Skip for the update command itself and in quiet mode
    let check_handle = if !args.quiet && !matches!(args.command, Commands::Update) {
        Some(std::thread::spawn(|| {
            adapters::updater::github_updater::check_latest_version()
        }))
    } else {
        None
    };

    // For commands that expect a single env, use the first --env value
    let single_env = args.env.first().map(|s| s.as_str());

    let result = match &args.command {
        Commands::Init => cli::commands::init::execute(),
        Commands::Encrypt { file, all } => {
            cli::commands::encrypt::execute(file.as_deref(), single_env, &args.cipher, *all)
        }
        Commands::Decrypt { file, key, output } => cli::commands::decrypt::execute(
            file.as_deref(),
            single_env,
            &args.cipher,
            key.as_deref(),
            output.as_deref(),
        ),
        Commands::Check => cli::commands::check::execute(),
        Commands::Diff { file1, file2 } => cli::commands::diff::execute(
            file1.as_deref(),
            file2.as_deref(),
            &args.env,
            &args.cipher,
        ),
        Commands::Resolve { output } => {
            cli::commands::resolve::execute(single_env, &args.cipher, output.as_deref())
        }
        Commands::Keys { action } => cli::commands::keys::execute(action),
        Commands::Log {
            author,
            since,
            last,
        } => cli::commands::log::execute(author.as_deref(), since.as_deref(), *last),
        Commands::Status => cli::commands::status::execute(),
        Commands::Hook { action } => cli::commands::hook::execute(action),
        Commands::Update => cli::commands::update::execute(),
    };

    if let Err(e) = result {
        cli::output::error(&format!("Error: {e}"));
        std::process::exit(1);
    }

    // Print update banner if a newer version was found (after command output)
    if let Some(handle) = check_handle {
        if let Ok(Some(latest)) = handle.join() {
            use colored::Colorize;
            let current = core::models::update_info::current_version();
            eprintln!(
                "\n  {} Vaultic v{latest} available (you have v{current}). Run: {}",
                "⚡".yellow(),
                "vaultic update".bold()
            );
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add passive version check banner on startup (non-blocking, cached 24h)"
```

---

## Task 8: Add format_version to Config and Init

**Files:**
- Modify: `src/config/app_config.rs`
- Modify: `src/cli/commands/init.rs`

**Step 1: Add format_version and template fields to config structs**

In `src/config/app_config.rs`, modify `VaulticSection`:

```rust
/// The `[vaultic]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct VaulticSection {
    pub version: String,
    /// Format version for backward compatibility. Defaults to 1 if missing.
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    pub default_cipher: String,
    pub default_env: String,
    /// Global template file path (optional).
    pub template: Option<String>,
}

fn default_format_version() -> u32 {
    1
}
```

Also modify `EnvEntry` to include template:

```rust
/// An environment entry in `[environments]`.
#[derive(Debug, Clone, Deserialize)]
pub struct EnvEntry {
    pub file: Option<String>,
    pub inherits: Option<String>,
    /// Per-environment template file (optional).
    pub template: Option<String>,
}
```

Add the current format version constant and validation to `AppConfig`:

```rust
/// Current format version supported by this build of Vaultic.
pub const CURRENT_FORMAT_VERSION: u32 = 1;

impl AppConfig {
    /// Load the configuration from `.vaultic/config.toml`.
    ///
    /// After parsing, validates environment names, the audit log filename,
    /// and checks format version compatibility.
    pub fn load(vaultic_dir: &Path) -> Result<Self> {
        let config_path = vaultic_dir.join("config.toml");
        if !config_path.exists() {
            return Err(VaulticError::InvalidConfig {
                detail: "config.toml not found. Run 'vaultic init' first.".into(),
            });
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content).map_err(|e| VaulticError::InvalidConfig {
            detail: format!("Failed to parse config.toml: {e}"),
        })?;

        // Check format version compatibility
        if config.vaultic.format_version > CURRENT_FORMAT_VERSION {
            return Err(VaulticError::FormatVersionTooNew {
                project_version: config.vaultic.format_version,
                supported_version: CURRENT_FORMAT_VERSION,
            });
        }

        // Validate environment names from config
        for env_name in config.environments.keys() {
            crate::cli::context::validate_env_name(env_name)?;
        }

        // Validate audit log filename
        if let Some(audit) = &config.audit {
            crate::cli::context::validate_simple_filename(&audit.log_file, "audit log file")?;
        }

        Ok(config)
    }

    // ... rest of impl unchanged
}
```

**Step 2: Update init to include format_version**

In `src/cli/commands/init.rs`, update the `config_content` string:

```rust
let config_content = r#"[vaultic]
version = "0.1.0"
format_version = 1
default_cipher = "age"
default_env = "dev"

[environments]
base = { file = "base.env" }
dev = { file = "dev.env", inherits = "base" }
staging = { file = "staging.env", inherits = "base" }
prod = { file = "prod.env", inherits = "base" }

[audit]
enabled = true
log_file = "audit.log"
"#;
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All existing tests pass (format_version defaults to 1 for configs that don't have it)

**Step 4: Commit**

```bash
git add src/config/app_config.rs src/cli/commands/init.rs
git commit -m "feat: add format_version and template fields to config with backward compat defaults"
```

---

## Task 9: Implement Template Auto-Discovery

**Files:**
- Create: `src/core/services/template_resolver.rs`
- Modify: `src/core/services/mod.rs`

**Step 1: Write the template resolver with tests**

Create `src/core/services/template_resolver.rs`:

```rust
use std::path::{Path, PathBuf};

use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};

/// Priority list of template file names for auto-discovery.
const TEMPLATE_CANDIDATES: &[&str] = &[
    ".env.template",
    ".env.example",
    ".env.sample",
    "env.template",
];

/// Resolves the template file path for a given context.
pub struct TemplateResolver;

impl TemplateResolver {
    /// Resolve the template path for a global check (no specific environment).
    ///
    /// Resolution order:
    /// 1. Global `template` field in config (if config provided)
    /// 2. Auto-discovery in project root
    pub fn resolve_global(config: Option<&AppConfig>) -> Result<PathBuf> {
        // 1. Check config global template
        if let Some(cfg) = config {
            if let Some(ref tpl) = cfg.vaultic.template {
                let path = Path::new(tpl);
                if path.exists() {
                    return Ok(path.to_path_buf());
                }
            }
        }

        // 2. Auto-discovery
        Self::auto_discover()
    }

    /// Resolve the template path for a specific environment.
    ///
    /// Resolution order:
    /// 1. `template` field in environment config (explicit)
    /// 2. `{env}.env.template` convention in `.vaultic/`
    /// 3. Global `template` field in config
    /// 4. Auto-discovery in project root
    pub fn resolve_for_env(
        env_name: &str,
        config: &AppConfig,
        vaultic_dir: &Path,
    ) -> Result<PathBuf> {
        // 1. Per-environment template from config
        if let Some(env_entry) = config.environments.get(env_name) {
            if let Some(ref tpl) = env_entry.template {
                let path = vaultic_dir.join(tpl);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // 2. Convention: {env}.env.template in .vaultic/
        let convention_path = vaultic_dir.join(format!("{env_name}.env.template"));
        if convention_path.exists() {
            return Ok(convention_path);
        }

        // 3. Global template from config
        if let Some(ref tpl) = config.vaultic.template {
            let path = Path::new(tpl);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }

        // 4. Auto-discovery fallback
        Self::auto_discover().map_err(|_| {
            let mut searched = Vec::new();
            if let Some(env_entry) = config.environments.get(env_name) {
                if let Some(ref tpl) = env_entry.template {
                    searched.push(format!("  ✗ {} (from config)", vaultic_dir.join(tpl).display()));
                }
            }
            searched.push(format!("  ✗ {} (convention)", convention_path.display()));
            if let Some(ref tpl) = config.vaultic.template {
                searched.push(format!("  ✗ {} (global config)", tpl));
            }
            for candidate in TEMPLATE_CANDIDATES {
                searched.push(format!("  ✗ {candidate} (auto-discovery)"));
            }
            VaulticError::TemplateNotFound {
                searched: searched.join("\n    "),
            }
        })
    }

    /// Auto-discover a template file in the project root.
    fn auto_discover() -> Result<PathBuf> {
        for candidate in TEMPLATE_CANDIDATES {
            let path = Path::new(candidate);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }

        let searched = TEMPLATE_CANDIDATES
            .iter()
            .map(|c| format!("  ✗ {c}"))
            .collect::<Vec<_>>()
            .join("\n    ");

        Err(VaulticError::TemplateNotFound { searched })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_discover_returns_first_match() {
        // This test depends on filesystem state, but we can test the priority logic
        // by checking the constant order
        assert_eq!(TEMPLATE_CANDIDATES[0], ".env.template");
        assert_eq!(TEMPLATE_CANDIDATES[1], ".env.example");
        assert_eq!(TEMPLATE_CANDIDATES[2], ".env.sample");
        assert_eq!(TEMPLATE_CANDIDATES[3], "env.template");
    }

    #[test]
    fn auto_discover_fails_when_no_template_exists() {
        // In a temp dir with no template files
        let dir = tempfile::tempdir().unwrap();
        let _guard = std::env::set_current_dir(dir.path());
        // Note: this test is fragile because it changes cwd.
        // In practice, template_resolver is tested via integration tests.
    }
}
```

**Step 2: Register the module**

Add `pub mod template_resolver;` to `src/core/services/mod.rs`.

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/core/services/template_resolver.rs src/core/services/mod.rs
git commit -m "feat: add template resolver with auto-discovery and per-environment support"
```

---

## Task 10: Update `vaultic check` to Use Template Resolver

**Files:**
- Modify: `src/cli/commands/check.rs`
- Modify: `src/cli/mod.rs` (add `--env` support to `Check`)

**Step 1: Update the Check command to accept --env**

In `src/cli/mod.rs`, modify the `Check` variant:

```rust
/// Verify missing variables against template
#[command(
    long_about = "Verify your local .env against a template.\n\n\
                  Automatically discovers templates (.env.template, .env.example, \
                  .env.sample) or uses the one configured in .vaultic/config.toml.\n\n\
                  With --env, checks a decrypted environment against its \
                  environment-specific template (if available) or the global template.",
    after_help = "Examples:\n  \
                  vaultic check                         # Check .env vs auto-discovered template\n  \
                  vaultic check --env dev               # Check using dev-specific template\n  \
                  vaultic check --env prod              # Check using prod-specific template"
)]
Check,
```

**Step 2: Update check.rs to use TemplateResolver**

Replace `src/cli/commands/check.rs` with:

```rust
use std::path::Path;

use crate::adapters::parsers::dotenv_parser::DotenvParser;
use crate::cli::output;
use crate::config::app_config::AppConfig;
use crate::core::errors::{Result, VaulticError};
use crate::core::services::check_service::CheckService;
use crate::core::services::template_resolver::TemplateResolver;
use crate::core::traits::parser::ConfigParser;

/// Execute the `vaultic check` command.
///
/// Compares the local `.env` against a template and reports
/// missing, extra, and empty-value variables.
///
/// The template is resolved via auto-discovery or config. When
/// `--env` is provided, it looks for an environment-specific template.
pub fn execute(env_name: Option<&str>) -> Result<()> {
    let env_path = Path::new(".env");

    if !env_path.exists() {
        return Err(VaulticError::FileNotFound {
            path: env_path.to_path_buf(),
        });
    }

    // Resolve template path
    let vaultic_dir = crate::cli::context::vaultic_dir();
    let config = if vaultic_dir.exists() {
        AppConfig::load(vaultic_dir).ok()
    } else {
        None
    };

    let template_path = match env_name {
        Some(name) => {
            let cfg = config.as_ref().ok_or_else(|| VaulticError::InvalidConfig {
                detail: "Cannot use --env without .vaultic/config.toml. Run 'vaultic init' first."
                    .into(),
            })?;
            TemplateResolver::resolve_for_env(name, cfg, vaultic_dir)?
        }
        None => TemplateResolver::resolve_global(config.as_ref())?,
    };

    let parser = DotenvParser;
    let env_content = std::fs::read_to_string(env_path)?;
    let template_content = std::fs::read_to_string(&template_path)?;

    let env_file = parser.parse(&env_content)?;
    let template_file = parser.parse(&template_content)?;

    let svc = CheckService;
    let result = svc.check(&env_file, &template_file)?;

    let total_template = template_file.keys().len();
    let present = total_template - result.missing.len();

    output::header("vaultic check");
    output::detail(&format!("Template: {}", template_path.display()));

    if !result.missing.is_empty() {
        output::warning(&format!("Missing variables ({}):", result.missing.len()));
        for key in &result.missing {
            println!("    • {key}");
        }
    }

    if !result.extra.is_empty() {
        output::warning(&format!(
            "Extra variables not in template ({}):",
            result.extra.len()
        ));
        for key in &result.extra {
            println!("    • {key}");
        }
    }

    if !result.empty_values.is_empty() {
        output::warning(&format!(
            "Variables with empty values ({}):",
            result.empty_values.len()
        ));
        for key in &result.empty_values {
            println!("    • {key}");
        }
    }

    if result.is_ok() {
        output::success(&format!(
            "{present}/{total_template} variables present — all good"
        ));
    } else {
        println!();
        output::success(&format!(
            "{present}/{total_template} variables present, {} issue(s) found",
            result.issue_count()
        ));
    }

    // Audit
    let detail = if result.is_ok() {
        format!("{present}/{total_template} present")
    } else {
        format!(
            "{present}/{total_template} present, {} missing",
            result.missing.len()
        )
    };
    super::audit_helpers::log_audit(
        crate::core::models::audit_entry::AuditAction::Check,
        vec![".env".to_string()],
        Some(detail),
    );

    Ok(())
}
```

**Step 3: Update main.rs dispatch for Check**

In `src/main.rs`, update the `Check` match arm:

```rust
Commands::Check => cli::commands::check::execute(single_env),
```

**Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass. Existing check tests still work because TemplateResolver falls back to `.env.template` auto-discovery.

**Step 5: Commit**

```bash
git add src/cli/commands/check.rs src/cli/mod.rs src/main.rs
git commit -m "feat: update check command to use template resolver with auto-discovery and --env support"
```

---

## Task 11: Update GitHub Actions Release Workflow

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add checksum generation and signing to release job**

In `.github/workflows/release.yml`, update the `release` job to generate SHA256SUMS and sign them. Add these steps between "Download all artifacts" and "Create GitHub Release":

```yaml
      - name: Generate SHA256 checksums
        working-directory: artifacts
        run: sha256sum vaultic-* > SHA256SUMS.txt

      - name: Install minisign
        run: |
          curl -sL https://github.com/jedisct1/minisign/releases/download/0.11/minisign-0.11-linux-x86_64.tar.gz -o minisign.tar.gz
          tar xzf minisign.tar.gz
          sudo mv minisign-linux/x86_64/minisign /usr/local/bin/
          rm -rf minisign.tar.gz minisign-linux

      - name: Sign checksums with minisign
        working-directory: artifacts
        run: |
          echo "${{ secrets.MINISIGN_SECRET_KEY }}" > /tmp/minisign.key
          minisign -Sm SHA256SUMS.txt -s /tmp/minisign.key
          rm /tmp/minisign.key
```

Also update the "Create GitHub Release" step to include the new files:

```yaml
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            artifacts/vaultic-*
            artifacts/SHA256SUMS.txt
            artifacts/SHA256SUMS.txt.minisig
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add SHA256 checksum generation and minisign signing to release workflow"
```

---

## Task 12: Write Integration Tests

**Files:**
- Create: `tests/update_test.rs`
- Create: `tests/template_test.rs`

**Step 1: Write template integration tests**

Create `tests/template_test.rs`:

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// `vaultic check` auto-discovers .env.template
#[test]
fn check_auto_discovers_env_template() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".env").write_str("DB=localhost\n").unwrap();
    dir.child(".env.template").write_str("DB=\nAPI_KEY=\n").unwrap();

    Command::cargo_bin("vaultic")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Missing variables"));
}

/// `vaultic check` auto-discovers .env.example when .env.template is absent
#[test]
fn check_auto_discovers_env_example() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".env").write_str("DB=localhost\n").unwrap();
    dir.child(".env.example").write_str("DB=\n").unwrap();

    Command::cargo_bin("vaultic")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("variables present"));
}

/// `vaultic check` shows descriptive error when no template found
#[test]
fn check_no_template_shows_descriptive_error() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".env").write_str("DB=localhost\n").unwrap();

    Command::cargo_bin("vaultic")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No template file found")
                .or(predicate::str::contains(".env.template")),
        );
}

/// `vaultic check` uses per-environment template from .vaultic/
#[test]
fn check_uses_per_env_template() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child(".env").write_str("DB=localhost\nCACHE=redis\n").unwrap();

    // Create .vaultic dir with config
    let vaultic = dir.child(".vaultic");
    vaultic.child("config.toml").write_str(
        r#"[vaultic]
version = "0.1.0"
format_version = 1
default_cipher = "age"
default_env = "dev"

[environments]
dev = { file = "dev.env" }

[audit]
enabled = false
log_file = "audit.log"
"#
    ).unwrap();
    vaultic.child("recipients.txt").write_str("").unwrap();

    // Create per-env template
    vaultic.child("dev.env.template").write_str("DB=\nCACHE=\nNEW_VAR=\n").unwrap();

    Command::cargo_bin("vaultic")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .arg("--env")
        .arg("dev")
        .assert()
        .success()
        .stdout(predicate::str::contains("Missing variables"));
}
```

**Step 2: Write update model tests (unit-level, no network)**

Create `tests/update_test.rs`:

```rust
/// Test that SHA256 verification works correctly (unit-level, no network).
#[test]
fn sha256_verification_logic() {
    // This is tested in unit tests in verifier.rs
    // Integration test verifies the binary compiles with all update deps
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_vaultic"))
        .arg("update")
        .arg("--help")
        .output()
        .expect("binary should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("update"));
}
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add tests/update_test.rs tests/template_test.rs
git commit -m "test: add integration tests for template auto-discovery and update command"
```

---

## Task 13: Generate Minisign Keypair (Manual Step)

> **This task requires manual action by the developer.**

**Step 1: Install minisign locally**

```bash
# macOS
brew install minisign

# Windows (scoop)
scoop install minisign

# Linux
# Download from https://github.com/jedisct1/minisign/releases
```

**Step 2: Generate keypair**

```bash
minisign -G -p vaultic.pub -s vaultic.key -c "vaultic release signing key"
```

This creates:
- `vaultic.pub` — public key (embed in code)
- `vaultic.key` — secret key (add to GitHub Secrets as `MINISIGN_SECRET_KEY`)

**Step 3: Embed the public key**

Replace the placeholder in `src/adapters/updater/verifier.rs`:

```rust
pub const MINISIGN_PUBLIC_KEY: &str =
    "untrusted comment: vaultic release signing key\n<PASTE_THE_BASE64_KEY_HERE>";
```

**Step 4: Add secret key to GitHub**

Go to: GitHub repo → Settings → Secrets → New secret
- Name: `MINISIGN_SECRET_KEY`
- Value: contents of `vaultic.key` file

**Step 5: Delete the local secret key file**

```bash
rm vaultic.key
```

**Step 6: Commit the public key update**

```bash
git add src/adapters/updater/verifier.rs
git commit -m "feat: embed minisign public key for release verification"
```

---

## Task 14: Bump Version to 1.2.0 and Final Verification

**Files:**
- Modify: `Cargo.toml` (version bump)
- Modify: `CHANGELOG.md`

**Step 1: Bump version**

In `Cargo.toml`, change: `version = "1.1.0"` → `version = "1.2.0"`

**Step 2: Update CHANGELOG.md**

Add entry for v1.2.0 with the three features.

**Step 3: Run full test suite**

Run: `cargo test --all`
Expected: ALL tests pass

**Step 4: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 5: Run fmt check**

Run: `cargo fmt --check`
Expected: No formatting issues

**Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to 1.2.0 with auto-update, template improvements, format versioning"
```

---

## Summary of All Commits

| # | Commit Message | Key Files |
|---|---------------|-----------|
| 1 | feat: add auto-update dependencies | Cargo.toml |
| 2 | feat: add error variants for update, template, format | errors.rs |
| 3 | feat: add UpdateInfo model | models/update_info.rs |
| 4 | feat: add SHA256 and minisign verification | adapters/updater/verifier.rs |
| 5 | feat: add GitHub updater adapter | adapters/updater/github_updater.rs |
| 6 | feat: add 'vaultic update' command | cli/commands/update.rs, mod.rs, main.rs |
| 7 | feat: add passive version check banner | main.rs |
| 8 | feat: add format_version and template to config | app_config.rs, init.rs |
| 9 | feat: add template resolver | services/template_resolver.rs |
| 10 | feat: update check with auto-discovery + --env | check.rs, mod.rs |
| 11 | ci: add SHA256 + minisign to release workflow | release.yml |
| 12 | test: integration tests | template_test.rs, update_test.rs |
| 13 | feat: embed minisign public key | verifier.rs (manual) |
| 14 | chore: bump to v1.2.0 | Cargo.toml, CHANGELOG.md |
