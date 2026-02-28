use sha2::{Digest, Sha256};

use crate::core::errors::{Result, VaulticError};

/// Embedded minisign public key for verifying release signatures.
///
/// This key is generated once and the corresponding secret key is
/// stored in GitHub Secrets for CI signing.
///
/// Replace this placeholder with the real public key after running:
/// `minisign -G -p vaultic.pub -s vaultic.key`
pub const MINISIGN_PUBLIC_KEY: &str =
    "untrusted comment: minisign public key for vaultic\nRWTOPLACEHOLDER_REPLACE_WITH_REAL_KEY_AFTER_GENERATION";

/// Compute the SHA256 hex digest of the given bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify that the SHA256 hash of `binary_data` matches the expected hash
/// for `asset_name` found in `checksums_content` (SHA256SUMS.txt format).
///
/// SHA256SUMS.txt format: `<hex_hash>  <filename>` (two spaces between).
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
    let pk_line = MINISIGN_PUBLIC_KEY
        .lines()
        .nth(1)
        .unwrap_or(MINISIGN_PUBLIC_KEY);

    let pk =
        minisign_verify::PublicKey::from_base64(pk_line).map_err(|e| {
            VaulticError::UpdateVerificationFailed {
                reason: format!("Invalid embedded public key: {e}"),
            }
        })?;

    let sig_str = String::from_utf8_lossy(signature_content);
    let sig = minisign_verify::Signature::decode(&sig_str).map_err(|e| {
        VaulticError::UpdateVerificationFailed {
            reason: format!("Invalid signature file: {e}"),
        }
    })?;

    pk.verify(checksums_content, &sig, false).map_err(|e| {
        VaulticError::UpdateVerificationFailed {
            reason: format!(
                "Invalid signature\n\n  \
                 SHA256SUMS.txt signature does not match the embedded public key.\n  \
                 This could indicate the release has been tampered with.\n\n  \
                 Error: {e}"
            ),
        }
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
    fn verify_sha256_fails_when_asset_missing() {
        let data = b"binary content";
        let checksums = "abc123  other-file";
        let result = verify_sha256(data, "vaultic-linux-amd64", checksums);
        assert!(result.is_err());
    }
}
