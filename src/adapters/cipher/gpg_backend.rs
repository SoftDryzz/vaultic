use std::path::PathBuf;
use std::process::Command;

use crate::core::errors::{Result, VaulticError};
use crate::core::models::key_identity::KeyIdentity;
use crate::core::traits::cipher::CipherBackend;

/// GPG encryption backend that shells out to the system `gpg` binary.
///
/// Requires GPG to be installed on the system. This backend is intended
/// for enterprise environments that already use GPG infrastructure.
pub struct GpgBackend {
    /// Path to the gpg binary (defaults to "gpg").
    gpg_path: PathBuf,
}

impl GpgBackend {
    /// Create a new backend using the default `gpg` binary.
    pub fn new() -> Self {
        Self {
            gpg_path: PathBuf::from("gpg"),
        }
    }

    /// Create a new backend with a custom gpg binary path.
    pub fn with_path(gpg_path: PathBuf) -> Self {
        Self { gpg_path }
    }

    /// Check if GPG is available on the system.
    pub fn is_available(&self) -> bool {
        Command::new(&self.gpg_path)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// Run a gpg command and return stdout on success.
    fn run_gpg(&self, args: &[&str], stdin_data: Option<&[u8]>) -> Result<Vec<u8>> {
        let mut cmd = Command::new(&self.gpg_path);
        cmd.args(args);

        if let Some(data) = stdin_data {
            use std::io::Write;
            use std::process::Stdio;

            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Failed to run gpg: {e}"),
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(data)
                    .map_err(|e| VaulticError::EncryptionFailed {
                        reason: format!("Failed to write to gpg stdin: {e}"),
                    })?;
            }

            let output = child
                .wait_with_output()
                .map_err(|e| VaulticError::EncryptionFailed {
                    reason: format!("gpg process failed: {e}"),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VaulticError::EncryptionFailed {
                    reason: format!("gpg exited with error: {stderr}"),
                });
            }

            Ok(output.stdout)
        } else {
            let output = cmd.output().map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Failed to run gpg: {e}"),
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VaulticError::EncryptionFailed {
                    reason: format!("gpg exited with error: {stderr}"),
                });
            }

            Ok(output.stdout)
        }
    }
}

impl Default for GpgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CipherBackend for GpgBackend {
    fn encrypt(&self, plaintext: &[u8], recipients: &[KeyIdentity]) -> Result<Vec<u8>> {
        if recipients.is_empty() {
            return Err(VaulticError::EncryptionFailed {
                reason: "No recipients provided".into(),
            });
        }

        let mut args = vec![
            "--encrypt",
            "--armor",
            "--batch",
            "--yes",
            "--trust-model",
            "always",
        ];

        // Collect recipient flags
        let recipient_args: Vec<String> = recipients
            .iter()
            .flat_map(|ki| vec!["--recipient".to_string(), ki.public_key.clone()])
            .collect();
        let recipient_refs: Vec<&str> = recipient_args.iter().map(|s| s.as_str()).collect();
        args.extend_from_slice(&recipient_refs);

        self.run_gpg(&args, Some(plaintext))
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let args = ["--decrypt", "--batch", "--yes"];

        self.run_gpg(&args, Some(ciphertext))
            .map_err(|_| VaulticError::DecryptionNoKey)
    }

    fn name(&self) -> &str {
        "gpg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpg_backend_has_correct_name() {
        let backend = GpgBackend::new();
        assert_eq!(backend.name(), "gpg");
    }

    #[test]
    fn encrypt_no_recipients_fails() {
        let backend = GpgBackend::new();
        let result = backend.encrypt(b"data", &[]);
        assert!(result.is_err());
    }

    // Integration tests that require GPG installed are in tests/integration/
}
