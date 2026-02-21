# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> English | **[Español](docs/CHANGELOG.es.md)**

## [Unreleased]

### Added

- `vaultic encrypt --all`: re-encrypt all environments for current recipients (key rotation, recipient changes)
- `vaultic decrypt --key <path>`: specify a custom private key location instead of the default
- `--quiet` / `--verbose` flags: suppress non-error output or show detailed information across all commands
- `--config <path>` flag: use a custom vaultic directory instead of the default `.vaultic/`
- GPG support in `decrypt_in_memory`: `vaultic resolve --cipher gpg` and `vaultic diff --cipher gpg` now work correctly
- `vaultic keys setup`: import existing age key from file (option 2), use existing GPG key from keyring (option 3, when GPG detected)
- Public key validation at `vaultic keys add`: validates age keys as `x25519::Recipient`, accepts GPG fingerprints and email identifiers
- SHA-256 `state_hash` in audit log: encrypt and decrypt operations now record the hash of the resulting file for integrity verification
- "Your key" section in `vaultic status`: shows private key location, public key, and whether you are in the recipients list
- GPG keyring detection during `vaultic init`: when no age key exists but GPG is available, offers a choice between age and GPG
- Input validation: environment names restricted to `[a-zA-Z0-9_-]` to prevent path traversal; audit log filename validated against path separators

### Fixed

- `truncate_key` no longer panics on non-ASCII characters (e.g. GPG identities with names like "María")
- `vaultic log` now shows the author column as specified in the documentation
- Hook commands now log proper `HookInstall`/`HookUninstall` audit actions instead of `Init`

## [0.5.0-alpha] - 2026-02-21

### Added

- `JsonAuditLogger`: append-only JSON lines logger with filtered queries by author and date
- Audit wiring: all commands (init, encrypt, decrypt, keys, resolve, check, diff) now record audit entries
- `audit_helpers` module: shared git identity resolution and non-blocking audit logging
- `vaultic log`: display audit history with `--author`, `--since`, and `--last N` filters
- `vaultic status`: full project dashboard showing config, recipients, encrypted environments, local state, and audit status
- `vaultic hook install/uninstall`: git pre-commit hook that blocks plaintext `.env` files from being committed
- `git_hook` adapter: safe install/uninstall with foreign hook detection via marker comments
- Removed global `#![allow(dead_code)]` — all unused items now have targeted annotations
- SECURITY.md: encryption model, incident response, vulnerability reporting (English + Spanish)
- CONTRIBUTING.md: contributor agreement for dual licensing, development guide (English + Spanish)
- COMMERCIAL.md: dual licensing FAQ for organizations (English + Spanish)
- 16 new unit tests (9 audit logger, 7 git hook)
- 14 new integration tests (audit, log, status, hook commands)

## [0.4.0-alpha] - 2026-02-20

### Added

- `EnvResolver`: multi-level environment inheritance with merge logic (overlay wins over base) and circular dependency detection
- `AppConfig::load()`: read and parse `.vaultic/config.toml` with environment definitions
- `vaultic resolve --env <env>`: resolve full inheritance chain, decrypt layers in memory, and write merged `.env`
- `vaultic diff --env dev --env prod`: compare two resolved environments side by side
- `decrypt_to_bytes` on `EncryptionService`: in-memory decryption without disk writes
- Repeatable `--env` flag: supports multiple values for environment comparison
- 13 new unit tests (env resolver merge, chain building, cycle detection)
- 6 integration tests (resolve command, environment diff)

## [0.3.0-alpha] - 2026-02-20

### Added

- Dotenv parser (`DotenvParser`): parse and serialize `.env` files preserving comments, blank lines, and ordering
- `Line` enum model (`Entry`/`Comment`/`Blank`) for lossless file round-trips
- `DiffService`: compare two secret files detecting added, removed, and modified variables
- `CheckService`: validate local `.env` against `.env.template` reporting missing, extra, and empty-value variables
- `vaultic check`: CLI command with colored output for template validation
- `vaultic diff <file1> <file2>`: CLI command with formatted table showing variable differences
- 27 unit tests (dotenv parser, diff service, check service)
- 11 integration tests (check and diff CLI commands with error paths)

## [0.2.0-alpha] - 2026-02-20

### Added

- Age encryption backend (`AgeBackend`): X25519 + ChaCha20-Poly1305 with ASCII-armored output
- GPG encryption backend (`GpgBackend`): shell-based integration with system GPG
- File-based key store (`FileKeyStore`): manage recipients via `.vaultic/recipients.txt`
- `EncryptionService`: orchestrates cipher backend + key store for file encryption/decryption
- `KeyService`: manages recipient keys through the key store
- `vaultic init`: interactive project setup with key detection and generation
- `vaultic encrypt`: encrypt files for all authorized recipients
- `vaultic decrypt`: decrypt files using the local private key
- `vaultic keys setup`: interactive key generation for new users
- `vaultic keys add/list/remove`: manage authorized recipients
- 15 unit tests (age backend, gpg backend, file key store)
- 12 integration tests (init, encrypt, decrypt, keys, error paths)

## [0.1.0-alpha] - 2026-02-19

### Added

- Hexagonal architecture: `core/`, `adapters/`, `cli/`, `config/` layers
- Domain models: `SecretFile`, `SecretEntry`, `Environment`, `KeyIdentity`, `AuditEntry`, `DiffResult`
- Core traits (ports): `CipherBackend`, `ConfigParser`, `KeyStore`, `AuditLogger`
- Service signatures: `EncryptionService`, `DiffService`, `CheckService`, `EnvResolver`, `KeyService`
- Typed error handling with `VaulticError` enum (11 variants)
- Full CLI parsing with clap: 10 commands + global flags
- Colored output helpers (`success`, `warning`, `error`, `header`)
- CI pipeline: fmt + clippy + test on Linux, macOS, Windows
- Release pipeline: cross-platform build + crates.io publish
- AGPL-3.0 License
- README with badges, installation, quick start, and command reference

[Unreleased]: https://github.com/SoftDryzz/vaultic/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/SoftDryzz/vaultic/releases/tag/v0.1.0-alpha
