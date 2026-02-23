# Security Policy

> English | **[Español](docs/SECURITY.es.md)**

## Encryption Model

Vaultic uses asymmetric cryptography (public/private key pairs):

- **age**: X25519 key agreement + ChaCha20-Poly1305 (default, recommended)
- **GPG**: depends on user configuration (RSA/ECC)

Each file is encrypted for N recipients. Only holders of a matching private key can decrypt.

## What Is Safe to Publish

| File | Safe in public repo? | Reason |
|------|----------------------|--------|
| `*.env.enc` | Yes | Encrypted, unreadable without private key |
| `recipients.txt` | Yes | Public keys only (used for encryption) |
| `config.toml` | Yes | Configuration metadata, no secrets |
| `audit.log` | Yes | Operation metadata only, no values |
| `.env` | **NEVER** | Plaintext secrets |
| `keys.txt` / private keys | **NEVER** | Private key material |

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x.x (current) | Yes |
| 0.x.x | No |

## Reporting a Vulnerability

If you discover a security vulnerability in Vaultic, please report it responsibly.

**DO NOT** open a public issue for security vulnerabilities.

**Email:** security@softdryzz.com

We will acknowledge receipt within 48 hours and aim to provide an initial assessment within 5 business days.

## Incident Response

### Leaked plaintext `.env` file

1. **Rotate ALL secrets immediately** (API keys, passwords, tokens)
2. Remove the file from Git history using `git filter-branch` or [BFG Repo-Cleaner](https://rtyley.github.io/bfg-repo-cleaner/)
3. Re-encrypt with new values: `vaultic encrypt --env <env>`
4. Audit access logs for any unauthorized usage

### Compromised private key

1. Remove the recipient: `vaultic keys remove <key>`
2. Generate a new key: `vaultic keys setup`
3. Re-encrypt all environments: `vaultic encrypt --all`
4. Rotate any secrets that were accessible with the compromised key
5. Previously encrypted files in Git history remain at risk — rotate affected secrets

### Team member departure

1. Remove their public key: `vaultic keys remove <key>`
2. Re-encrypt all environments: `vaultic encrypt --all` (ensures new encryptions exclude the removed key)
3. Rotate sensitive secrets (production API keys, database passwords, signing keys)

## Security Design Principles

- **No plaintext on disk during resolution**: `vaultic resolve` decrypts layers in memory and writes only the final merged result
- **No network calls**: Vaultic v1 operates entirely offline — no telemetry, no cloud dependencies
- **No secret values in logs**: the audit log records operations and metadata, never variable values
- **Encryption is always asymmetric**: secrets are encrypted to specific recipients, never with symmetric passwords
- **Integrity verification**: encrypt and decrypt operations record a SHA-256 hash of the resulting file in the audit log, enabling tamper detection
- **Recipient key validation**: public keys are validated at add time (age Bech32 format, GPG fingerprint format) to prevent typos from causing silent failures
- **Input sanitization**: environment names and config file paths are validated against path traversal patterns to prevent a compromised `config.toml` from writing outside `.vaultic/`
