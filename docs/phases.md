# Vaultic — Development Phases

Overview of each development phase, its scope, and current status.
For the detailed architectural specification, see the project's internal documentation.

> English | **[Español](phases.es.md)**

---

## Phase 1 — Foundation ✅

Establishes the project skeleton and architectural boundaries.

- **Hexagonal architecture** scaffolded: `core/` (domain), `adapters/` (implementations), `cli/` (presentation), `config/`
- **Domain layer** defined: models, traits (ports), service signatures, and typed error handling
- **CLI parsing** with clap: all 10 commands registered with global flags (`--cipher`, `--env`, `--verbose`)
- **CI/CD pipelines** configured: format + lint + test on three platforms; release workflow for binaries and crates.io
- **Project metadata**: README with badges, AGPL-3.0 license, `.gitignore`

---

## Phase 2 — Encryption ✅

Implements the core encryption engine with dual backend support.

- **Age backend** (`AgeBackend`): encrypt/decrypt using X25519 + ChaCha20-Poly1305 with ASCII-armored output
- **GPG backend** (`GpgBackend`): shell-based integration with system GPG, no C dependencies
- **Strategy pattern** operational: select backend via `--cipher age|gpg` flag, same service orchestrates both
- **Key management**: `vaultic keys setup/add/list/remove` — interactive onboarding + recipient management
- **`vaultic init`** creates `.vaultic/` directory structure with interactive key detection and generation
- **27 tests**: 15 unit (backends + key store) + 12 integration (full CLI workflows)

---

## Phase 3 — Diff and Check ✅

Adds variable detection and file comparison capabilities.

- **Dotenv parser** (`DotenvParser`): parse and serialize `.env` files preserving comments, blank lines, and order with `Line` enum (`Entry`/`Comment`/`Blank`)
- **Check command**: compare local `.env` against `.env.template` — report missing, extra, and empty variables with summary counts
- **Diff command**: compare two secret files showing added, removed, and modified keys in a formatted table
- **Colored output**: formatted tables and status indicators for diff/check results
- **38 tests**: 27 unit (dotenv parser + diff service + check service) + 11 integration (check and diff CLI commands)

---

## Phase 4 — Multi-environment and Inheritance ✅

Enables layered environment management with smart resolution.

- **Environment resolver** (`EnvResolver`): multi-level merge (base → shared → dev) with overlay-wins semantics and 13 unit tests
- **Config-driven environments**: `AppConfig::load()` reads environment definitions and inheritance chains from `config.toml`
- **`vaultic resolve --env <env>`**: decrypt layers in memory, merge from root to leaf, write resolved `.env`
- **Cross-environment diff**: `vaultic diff --env dev --env prod` decrypts and resolves both environments before comparing
- **Circular inheritance detection**: error with clear diagnostic when cycles are found (e.g. `dev → staging → dev`)
- **In-memory decryption**: `decrypt_to_bytes` avoids temporary files during resolution
- **Repeatable `--env` flag**: `Vec<String>` allows `--env dev --env prod` syntax
- **25 tests**: 13 unit (resolver merge, chain, cycles) + 6 integration (resolve, env-diff) + 6 existing truncate tests

---

## Phase 5 — Audit and Polish ✅

Completes the feature set with audit logging, status reporting, and UX polish.

- **Audit logger** (`JsonAuditLogger`): append-only JSON lines in `.vaultic/audit.log` with filtered queries by author and date
- **Audit wiring**: all commands record audit entries via shared `audit_helpers` module with non-blocking logging and git identity resolution
- **`vaultic log`** with filters: `--author` (case-insensitive name/email), `--since` (ISO 8601), `--last N`
- **`vaultic status`**: full project dashboard — config, recipients, encrypted environments with file sizes, local state (.env, template, gitignore), audit entry count
- **Git pre-commit hook**: `vaultic hook install/uninstall` — blocks plaintext `.env` files from being committed, safe install with foreign hook detection
- **Descriptive error messages**: every error includes cause, context, and suggested next step
- **30 new tests**: 16 unit (9 audit logger + 7 git hook) + 14 integration (audit, log, status, hook)
- Removed global `#![allow(dead_code)]` — targeted annotations on reserved API surface

---

## Post-Phase 5 — v1.0 Hardening 🔲

Bug fixes, missing features, input validation, and polish toward the v1.0 release.

- **Bug fixes**: `truncate_key` panic on non-ASCII, missing `--all`/`--key` flags, hook audit actions, log author column
- **CLI flags wired**: `--quiet`/`--verbose` across all commands, `--config` for custom vaultic directory
- **GPG parity**: GPG backend now works in resolve, diff, init detection, and keys setup
- **Key management**: `keys setup` offers import existing age key and GPG keyring options; `keys add` validates key format before storing
- **Audit integrity**: SHA-256 `state_hash` recorded for encrypt/decrypt operations
- **Status dashboard**: "Your key" section showing private key location, public key, and recipient list status
- **Input validation**: environment names restricted to `[a-zA-Z0-9_-]`, audit log filename validated against path separators — prevents path traversal from CLI and compromised config files
- **Error message quality**: cause + solution pattern for all error variants (in progress)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Completed |
| 🔲 | Planned |
