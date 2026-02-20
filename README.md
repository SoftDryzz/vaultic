# Vaultic

[![CI](https://github.com/SoftDryzz/vaultic/workflows/CI/badge.svg)](https://github.com/SoftDryzz/vaultic/actions)
[![crates.io](https://img.shields.io/crates/v/vaultic.svg)](https://crates.io/crates/vaultic)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

> English | **[Español](docs/README.es.md)**

**Secure your secrets. Sync your team. Trust your configs.**

Vaultic is a CLI tool for managing secrets and configuration files securely across development teams. It encrypts your sensitive files, syncs them via Git, detects missing variables, and audits every change.

## Why Vaultic?

- **Strong encryption** — age or GPG, your choice
- **Detects problems** — missing variables, out-of-sync configs
- **Multi-environment** — dev/staging/prod with smart inheritance
- **Audit trail** — who changed what, when
- **Zero cloud** — everything local + Git, no external dependencies
- **Extensible** — designed to support .env, .toml, .yaml, .json

## Installation

### With Cargo (requires Rust)

```bash
cargo install vaultic
```

### Precompiled binaries

Download from [Releases](https://github.com/SoftDryzz/vaultic/releases) for Windows, Linux, or macOS.

## Quick Start

```bash
# 1. Initialize in your project
cd my-project
vaultic init

# 2. Encrypt your secrets
vaultic encrypt .env --env dev

# 3. Commit the encrypted file (safe)
git add .vaultic/
git commit -m "feat: add encrypted secrets"

# 4. Another dev clones and decrypts
vaultic decrypt --env dev
```

## Commands

| Command | Description |
|---------|-------------|
| `vaultic init` | Initialize Vaultic in the current project |
| `vaultic encrypt [file]` | Encrypt secret files |
| `vaultic decrypt [file]` | Decrypt secret files |
| `vaultic check` | Verify missing variables against template |
| `vaultic diff` | Compare secret files or environments |
| `vaultic resolve --env <env>` | Generate resolved file with inheritance |
| `vaultic keys setup` | Generate or import a key |
| `vaultic keys add <key>` | Add a recipient |
| `vaultic keys list` | List authorized recipients |
| `vaultic keys remove <key>` | Remove a recipient |
| `vaultic log` | Show operation history |
| `vaultic status` | Show full project status |
| `vaultic hook install` | Install git pre-commit hook |

## Security

Encrypted `.enc` files use asymmetric cryptography. Only authorized recipients can decrypt them with their private key. Public keys in the repository are only used for encryption and pose no risk.

See [SECURITY.md](SECURITY.md) for the full security policy.

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).

Commercial licensing is available for organizations that require alternative terms. Contact: legal@softdryzz.com
