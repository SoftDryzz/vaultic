# Security Policy

> English | **[Español](docs/SECURITY.es.md)**

## Encryption Model

Vaultic uses asymmetric cryptography (public/private key pairs):

- **age**: X25519 key agreement + ChaCha20-Poly1305 (default, recommended)
- **GPG**: depends on user configuration (RSA/ECC)

Each file is encrypted for N recipients. Only holders of a matching private key can decrypt.

### How Multi-Recipient Encryption Works

When you run `vaultic encrypt`, the file is **not** encrypted once per recipient. Instead:

1. A random **file key** is generated (one-time use, 256-bit)
2. The file contents are encrypted **once** with this file key (ChaCha20-Poly1305 — fast symmetric cipher)
3. The file key itself is then encrypted **separately for each recipient** using their public key (X25519 — asymmetric)

The resulting `.enc` file looks like this:

```
┌─────────────────────────────────────┐
│ Header                              │
│  → file key encrypted for Alice     │  ← only Alice's private key opens this
│  → file key encrypted for Bob       │  ← only Bob's private key opens this
│  → file key encrypted for Carol     │  ← only Carol's private key opens this
├─────────────────────────────────────┤
│ Body                                │
│  → file contents encrypted with     │
│     the file key (ChaCha20-Poly1305)│
└─────────────────────────────────────┘
```

**To decrypt**, a recipient:
1. Finds the header block that matches their public key
2. Decrypts the file key using their private key
3. Uses the file key to decrypt the body

This means:
- **Adding recipients does not increase file size significantly** — only the header grows (~140 bytes per recipient)
- **Each person decrypts independently** — no shared secrets, no key exchange between team members
- **Removing a recipient requires re-encryption** (`encrypt --all`) — the file must be re-encrypted with a new file key that excludes the removed recipient

### Why public keys in the repo are safe

A public key can **only encrypt** — it cannot decrypt anything. Even if an attacker has all the public keys and all the `.enc` files, they cannot recover any secrets. They would need a private key, which never leaves the owner's machine.

Think of a public key as a mailbox slot: anyone can drop a letter in (encrypt), but only the owner with their key can open it and read the contents (decrypt).

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
