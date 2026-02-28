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
    let _ = serde_json::to_string(&cache).map(|json| std::fs::write(&path, json));
}

/// Fetch the latest release info from GitHub (quick check, 3s timeout).
///
/// Returns `Some(version_string)` if a newer version is available, `None` otherwise.
/// Never errors — returns `None` on any failure (network, parse, etc.).
pub fn check_latest_version() -> Option<String> {
    if is_cache_fresh() {
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
        let version_str = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
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
    let asset_name =
        current_platform_asset().ok_or_else(|| VaulticError::UnsupportedPlatform {
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

        let release: GitHubRelease =
            resp.json()
                .await
                .map_err(|e| VaulticError::UpdateCheckFailed {
                    reason: format!("Failed to parse GitHub response: {e}"),
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
