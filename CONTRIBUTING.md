# Contributing to Vaultic

> English | **[Español](docs/CONTRIBUTING.es.md)**

Thank you for your interest in contributing to Vaultic! This guide will help you get started.

## License and Contributor Agreement

Vaultic is distributed under a **dual license** model:

- **Open source**: [GNU Affero General Public License v3.0](LICENSE)
- **Commercial**: Available for organizations that need alternative terms

By submitting a pull request, you agree that:

1. Your contribution is licensed under **AGPLv3**, consistent with the project license
2. You grant **SoftDryzz** a perpetual, irrevocable, worldwide, royalty-free license to use, modify, and sublicense your contribution under any license — including the commercial license offered alongside AGPLv3
3. You have the legal right to make the contribution (it is your original work or you have authorization)

This agreement is necessary to maintain the dual licensing model. Without it, external contributions could not be included in the commercial version.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, latest)
- [age](https://github.com/FiloSottile/age) (for encryption tests)
- Git

### Setup

```bash
git clone https://github.com/SoftDryzz/vaultic.git
cd vaultic
cargo build
cargo test
```

### Verify Everything Works

```bash
cargo fmt --check       # Code formatting
cargo clippy -- -D warnings  # Linting
cargo test --all-features    # All tests
```

All three must pass before submitting a PR.

## Project Architecture

Vaultic follows a **hexagonal architecture** (Clean Architecture adapted for Rust):

```
src/
├── cli/          # Presentation layer (CLI commands, output formatting)
├── core/         # Domain layer (pure business logic, zero external deps)
│   ├── traits/   # Ports (interfaces): CipherBackend, ConfigParser, etc.
│   ├── models/   # Entities: SecretFile, Environment, DiffResult, etc.
│   ├── services/ # Use cases: EncryptionService, DiffService, etc.
│   └── errors.rs # Domain error types
├── adapters/     # Implementations: AgeBackend, DotenvParser, FileKeyStore
└── config/       # Application configuration (config.toml parsing)
```

**Key rule**: `core/` never imports from `adapters/` or `cli/`. Dependencies flow inward.

## Code Standards

1. **Rust idioms**: `Result<T>` for errors, `Option<T>` for absence, iterators over manual loops, exhaustive pattern matching
2. **No `unwrap()` in production**: only in tests. Use `?` to propagate errors
3. **No unnecessary `clone()`**: prefer references and lifetimes
4. **Documentation**: `///` doc comments on all public traits, structs, and functions
5. **Tests**: at least one test per public function in `core/`
6. **Modules**: if a file exceeds ~150 lines, consider splitting
7. **Error messages**: each error variant must provide enough context to diagnose without debugging

## Pull Request Process

1. **Fork** the repository and create a branch from `master`
2. **Write your code** following the standards above
3. **Add tests** for any new functionality
4. **Run the full check**:
   ```bash
   cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```
5. **Write a clear commit message** describing the change
6. **Open a PR** against `master` with:
   - A concise title (under 70 characters)
   - Description of what changes and why
   - Any relevant context or trade-offs

## What to Contribute

### Good First Contributions

- Bug fixes with clear reproduction steps
- Test coverage improvements
- Documentation corrections or clarifications
- Error message improvements (more context, clearer next steps)

### Larger Contributions

For features or architectural changes, **open an issue first** to discuss the approach. This avoids wasted effort if the direction doesn't align with the project roadmap.

### Areas of Interest

- Parser implementations for new formats (TOML, YAML, JSON) via the `ConfigParser` trait
- Improved error diagnostics and user-facing messages
- Platform-specific improvements (Windows, macOS, Linux)

## Reporting Issues

- **Bugs**: Include Vaultic version, OS, steps to reproduce, expected vs actual behavior
- **Features**: Describe the use case, not just the solution
- **Security**: See [SECURITY.md](SECURITY.md) — do NOT open public issues for vulnerabilities

## Questions?

Open a [GitHub Discussion](https://github.com/SoftDryzz/vaultic/discussions) or reach out at legal@softdryzz.com for licensing questions.
