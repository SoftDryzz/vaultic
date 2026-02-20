# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> English | **[Español](docs/CHANGELOG.es.md)**

## [Unreleased]

### Planned

- Age encryption backend (Phase 2)
- GPG encryption backend (Phase 2)
- Encrypt/decrypt commands operational (Phase 2)
- Key management: add, list, remove recipients (Phase 2)
- Dotenv parser with variable detection (Phase 3)
- Diff and check commands (Phase 3)
- Multi-environment resolution with inheritance (Phase 4)
- Audit log with JSON lines (Phase 5)
- Git pre-commit hook (Phase 5)

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
