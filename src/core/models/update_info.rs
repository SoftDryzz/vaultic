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
    /// Git tag name (e.g., "v1.2.0").
    pub tag_name: String,
    /// URL to the release page on GitHub.
    pub html_url: String,
    /// List of downloadable assets attached to the release.
    pub assets: Vec<GitHubAsset>,
}

/// A single asset in a GitHub Release.
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    /// Filename of the asset (e.g., "vaultic-linux-amd64").
    pub name: String,
    /// Direct download URL for the asset.
    pub browser_download_url: String,
}

/// Cached result of a version check, stored locally to avoid API spam.
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
    fn platform_asset_returns_value() {
        // On CI this runs across platforms — just check it doesn't panic
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
