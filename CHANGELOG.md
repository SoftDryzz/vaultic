# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> English | **[Español](docs/CHANGELOG.es.md)**

## [Unreleased]

### Planned

- Multi-environment resolution with inheritance (Phase 4)
- Audit log with JSON lines (Phase 5)
- Git pre-commit hook (Phase 5)

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
