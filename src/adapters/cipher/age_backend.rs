use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use age::secrecy::ExposeSecret;

use crate::core::errors::{Result, VaulticError};
use crate::core::models::key_identity::KeyIdentity;
use crate::core::traits::cipher::CipherBackend;

/// Age encryption backend using X25519 + ChaCha20-Poly1305.
///
/// Uses ASCII-armored output so encrypted files are text-friendly
/// and work well with Git.
pub struct AgeBackend {
    /// Path to the age identity (private key) file.
    identity_path: PathBuf,
}

impl AgeBackend {
    /// Create a new backend pointing to a specific identity file.
    pub fn new(identity_path: PathBuf) -> Self {
        Self { identity_path }
    }

    /// Default identity file location for the current platform.
    ///
    /// - Linux/macOS: `~/.config/age/keys.txt`
    /// - Windows: `%APPDATA%/age/keys.txt`
    pub fn default_identity_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| VaulticError::InvalidConfig {
            detail: "Could not determine config directory".into(),
        })?;
        Ok(config_dir.join("age").join("keys.txt"))
    }

    /// Generate a new age X25519 identity, save it to `path`,
    /// and return the public key string.
    pub fn generate_identity(path: &Path) -> Result<String> {
        let identity = age::x25519::Identity::generate();
        let public_key = identity.to_public().to_string();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let contents = format!(
            "# created: {created}\n# public key: {public_key}\n{}\n",
            identity.to_string().expose_secret()
        );
        std::fs::write(path, contents)?;

        Ok(public_key)
    }

    /// Read the public key from an existing identity file.
    pub fn read_public_key(path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path).map_err(|_| VaulticError::FileNotFound {
            path: path.to_path_buf(),
        })?;

        // Parse the "# public key: age1..." comment line
        for line in content.lines() {
            if let Some(key) = line.strip_prefix("# public key: ") {
                return Ok(key.trim().to_string());
            }
        }

        // Fallback: parse the identity and derive the public key
        let identity: age::x25519::Identity = content
            .lines()
            .find(|l| l.starts_with("AGE-SECRET-KEY-"))
            .ok_or_else(|| VaulticError::InvalidConfig {
                detail: format!("No secret key found in {}", path.display()),
            })?
            .parse()
            .map_err(|e| VaulticError::InvalidConfig {
                detail: format!("Invalid age key in {}: {e}", path.display()),
            })?;

        Ok(identity.to_public().to_string())
    }

    /// Parse recipient strings into age X25519 recipients.
    fn parse_recipients(keys: &[KeyIdentity]) -> Result<Vec<age::x25519::Recipient>> {
        keys.iter()
            .map(|ki| {
                ki.public_key
                    .parse::<age::x25519::Recipient>()
                    .map_err(|e: &str| VaulticError::EncryptionFailed {
                        reason: format!("Invalid recipient key '{}': {e}", ki.public_key),
                    })
            })
            .collect()
    }

    /// Load identities from the private key file.
    fn load_identities(&self) -> Result<Vec<Box<dyn age::Identity>>> {
        let path_str = self.identity_path.to_string_lossy().to_string();
        let identity_file =
            age::IdentityFile::from_file(path_str).map_err(|e| VaulticError::EncryptionFailed {
                reason: format!(
                    "Failed to read identity file '{}': {e}",
                    self.identity_path.display()
                ),
            })?;

        identity_file
            .into_identities()
            .map_err(|_| VaulticError::DecryptionNoKey)
    }
}

impl CipherBackend for AgeBackend {
    fn encrypt(&self, plaintext: &[u8], recipients: &[KeyIdentity]) -> Result<Vec<u8>> {
        if recipients.is_empty() {
            return Err(VaulticError::EncryptionFailed {
                reason: "No recipients provided".into(),
            });
        }

        let parsed = Self::parse_recipients(recipients)?;

        let encryptor =
            age::Encryptor::with_recipients(parsed.iter().map(|r| r as &dyn age::Recipient))
                .map_err(|e| VaulticError::EncryptionFailed {
                    reason: format!("{e}"),
                })?;

        // Encrypt with ASCII armor for Git-friendly output
        let mut output = Vec::new();
        let armored =
            age::armor::ArmoredWriter::wrap_output(&mut output, age::armor::Format::AsciiArmor)
                .map_err(|e| VaulticError::EncryptionFailed {
                    reason: format!("Armor writer failed: {e}"),
                })?;

        let mut writer =
            encryptor
                .wrap_output(armored)
                .map_err(|e| VaulticError::EncryptionFailed {
                    reason: format!("Encryption stream failed: {e}"),
                })?;

        writer
            .write_all(plaintext)
            .map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Write failed: {e}"),
            })?;

        let armored_writer = writer
            .finish()
            .map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Encryption finish failed: {e}"),
            })?;

        armored_writer
            .finish()
            .map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Armor finish failed: {e}"),
            })?;

        Ok(output)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let identities = self.load_identities()?;

        let armored_reader = age::armor::ArmoredReader::new(ciphertext);
        let decryptor =
            age::Decryptor::new(armored_reader).map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Invalid encrypted file: {e}"),
            })?;

        let mut reader = decryptor
            .decrypt(identities.iter().map(|i| i.as_ref()))
            .map_err(|_| VaulticError::DecryptionNoKey)?;

        let mut plaintext = Vec::new();
        reader
            .read_to_end(&mut plaintext)
            .map_err(|e| VaulticError::EncryptionFailed {
                reason: format!("Read decrypted data failed: {e}"),
            })?;

        Ok(plaintext)
    }

    fn name(&self) -> &str {
        "age"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_read_public_key() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("keys.txt");

        let public_key = AgeBackend::generate_identity(&key_path).unwrap();
        assert!(public_key.starts_with("age1"));

        let read_back = AgeBackend::read_public_key(&key_path).unwrap();
        assert_eq!(public_key, read_back);
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("keys.txt");

        let public_key = AgeBackend::generate_identity(&key_path).unwrap();
        let backend = AgeBackend::new(key_path);

        let recipient = KeyIdentity {
            public_key,
            label: None,
            added_at: None,
        };

        let plaintext = b"DATABASE_URL=postgres://localhost/mydb\nAPI_KEY=secret123";
        let ciphertext = backend.encrypt(plaintext, &[recipient]).unwrap();

        // Verify armored output
        let armored_str = String::from_utf8_lossy(&ciphertext);
        assert!(armored_str.contains("BEGIN AGE ENCRYPTED FILE"));

        let decrypted = backend.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_multiple_recipients() {
        let dir = tempfile::tempdir().unwrap();

        // Generate two identities
        let key1_path = dir.path().join("key1.txt");
        let key2_path = dir.path().join("key2.txt");
        let pub1 = AgeBackend::generate_identity(&key1_path).unwrap();
        let pub2 = AgeBackend::generate_identity(&key2_path).unwrap();

        let recipients = vec![
            KeyIdentity {
                public_key: pub1,
                label: Some("dev1".into()),
                added_at: None,
            },
            KeyIdentity {
                public_key: pub2,
                label: Some("dev2".into()),
                added_at: None,
            },
        ];

        // Encrypt with key1's backend
        let backend1 = AgeBackend::new(key1_path);
        let plaintext = b"SHARED_SECRET=multi_recipient_test";
        let ciphertext = backend1.encrypt(plaintext, &recipients).unwrap();

        // Both keys can decrypt
        let decrypted1 = backend1.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted1, plaintext);

        let backend2 = AgeBackend::new(key2_path);
        let decrypted2 = backend2.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted2, plaintext);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let dir = tempfile::tempdir().unwrap();

        let key1_path = dir.path().join("key1.txt");
        let key2_path = dir.path().join("key2.txt");
        let pub1 = AgeBackend::generate_identity(&key1_path).unwrap();
        let _pub2 = AgeBackend::generate_identity(&key2_path).unwrap();

        let recipient = KeyIdentity {
            public_key: pub1,
            label: None,
            added_at: None,
        };

        let backend1 = AgeBackend::new(key1_path);
        let ciphertext = backend1.encrypt(b"secret", &[recipient]).unwrap();

        // key2 cannot decrypt data encrypted only for key1
        let backend2 = AgeBackend::new(key2_path);
        let result = backend2.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_no_recipients_fails() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("keys.txt");
        AgeBackend::generate_identity(&key_path).unwrap();

        let backend = AgeBackend::new(key_path);
        let result = backend.encrypt(b"data", &[]);
        assert!(result.is_err());
    }
}
