# Design: Auto-Update + Template Detection + Backward Compatibility

**Date:** 2026-02-28
**Status:** Approved
**Version target:** v1.2.0

---

## Summary

Three interconnected features for Vaultic v1.2.0:

1. **Auto-Update** — Check for new versions via GitHub API, download with SHA256 + minisign verification, self-replace binary
2. **Template Detection** — Auto-discover template files, support per-environment templates
3. **Backward Compatibility** — Format versioning to ensure updates never break existing encrypted files

---

## Feature 1: Auto-Update with Verified Downloads

### Architecture

New files following the existing hexagonal architecture:

```
src/
  core/
    traits/updater.rs          # trait UpdateChecker (port)
    models/update_info.rs      # UpdateInfo, ReleaseAsset, VersionCompat
  adapters/
    updater/
      mod.rs
      github_updater.rs        # Implements UpdateChecker via GitHub API
      verifier.rs              # SHA256 + minisign verification
  cli/
    commands/update.rs         # `vaultic update` command
```

### Passive Version Check (on every command)

- Spawn a background thread at startup
- GET `https://api.github.com/repos/SoftDryzz/vaultic/releases/latest`
- Timeout: 3 seconds — silent failure if offline
- Compare `tag_name` against current version using `semver` crate
- If newer version available, print banner AFTER command output:
  ```
  ⚡ Vaultic v1.2.0 available (you have v1.1.0). Run: vaultic update
  ```
- Cache check result in `~/.config/vaultic/last_update_check.json`
- Only check once every 24 hours to avoid API spam
- Respect `--quiet` flag: suppress banner in quiet mode

### Explicit Update Command: `vaultic update`

Flow:

```
1. GET GitHub API → latest release metadata
2. Match asset for current platform:
   - vaultic-linux-amd64
   - vaultic-linux-arm64
   - vaultic-darwin-amd64
   - vaultic-darwin-arm64
   - vaultic-windows-amd64.exe
3. Download three files to temp directory:
   a. Binary artifact
   b. SHA256SUMS.txt
   c. SHA256SUMS.txt.minisig
4. Verify integrity:
   a. Compute SHA256 of downloaded binary
   b. Parse SHA256SUMS.txt, find line matching binary filename
   c. Compare computed hash == expected hash
   d. Verify minisign signature of SHA256SUMS.txt against embedded public key
5. If ALL checks pass → self_replace (atomic binary replacement)
6. Print success:
   ✓ Updated to v1.2.0
   Changelog: https://github.com/SoftDryzz/vaultic/releases/tag/v1.2.0
7. If ANY check fails → abort with descriptive error, original binary untouched
```

### Verification Architecture

```
                    ┌─────────────────────┐
                    │  GitHub Release      │
                    │                      │
                    │  vaultic-linux-amd64 │
                    │  SHA256SUMS.txt      │──── signed with minisign secret key (CI only)
                    │  SHA256SUMS.txt.minisig│
                    └───────────┬─────────┘
                                │ download
                    ┌───────────▼─────────┐
                    │  Local Verification  │
                    │                      │
                    │  1. SHA256(binary)    │
                    │     == SHA256SUMS     │
                    │                      │
                    │  2. minisign.verify(  │
                    │     SHA256SUMS.txt,   │
                    │     .minisig,         │
                    │     EMBEDDED_PUBKEY)  │
                    └───────────┬─────────┘
                                │ if both pass
                    ┌───────────▼─────────┐
                    │  self_replace()      │
                    │  atomic binary swap  │
                    └─────────────────────┘
```

### Minisign Key Management

- **Secret key**: stored in GitHub Secrets (`MINISIGN_SECRET_KEY`), used only in CI
- **Public key**: embedded in the binary as a compile-time constant
  ```rust
  const MINISIGN_PUBLIC_KEY: &str = "untrusted comment: minisign public key for vaultic\nRW...";
  ```
- Key generation (one-time): `minisign -G -p vaultic.pub -s vaultic.key`

### Platform Detection

```rust
fn current_platform_asset() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64")  => "vaultic-linux-amd64",
        ("linux", "aarch64") => "vaultic-linux-arm64",
        ("macos", "x86_64")  => "vaultic-darwin-amd64",
        ("macos", "aarch64") => "vaultic-darwin-arm64",
        ("windows", "x86_64") => "vaultic-windows-amd64.exe",
        _ => panic!("Unsupported platform"),
    }
}
```

### Error Messages

```
✗ Update verification failed: SHA256 mismatch

  Downloaded binary hash:  a1b2c3d4...
  Expected hash:           e5f6g7h8...

  The download may be corrupted or tampered with.

  Solutions:
    → Try again: vaultic update
    → Manual download: https://github.com/SoftDryzz/vaultic/releases/latest
    → Verify manually: sha256sum vaultic-linux-amd64
```

```
✗ Update verification failed: invalid signature

  SHA256SUMS.txt signature does not match the embedded public key.
  This could indicate the release has been tampered with.

  Solutions:
    → Report this at: https://github.com/SoftDryzz/vaultic/issues
    → Manual download and verify: https://github.com/SoftDryzz/vaultic/releases/latest
```

### GitHub Actions Changes

```yaml
# Addition to release.yml
- name: Install minisign
  run: |
    curl -sL https://github.com/jedisct1/minisign/releases/latest/download/minisign-linux-x86_64 -o minisign
    chmod +x minisign

- name: Generate checksums
  run: sha256sum vaultic-* > SHA256SUMS.txt

- name: Sign checksums
  run: echo "${{ secrets.MINISIGN_SECRET_KEY }}" | ./minisign -Sm SHA256SUMS.txt -s /dev/stdin

- name: Upload release artifacts
  uses: softprops/action-gh-release@v2
  with:
    files: |
      vaultic-*
      SHA256SUMS.txt
      SHA256SUMS.txt.minisig
```

### New Dependencies

```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
semver = "1"
minisign-verify = "0.2"
self_replace = "1"
tokio = { version = "1", features = ["rt", "macros"] }  # for async reqwest
```

Note: `tokio` is needed because `reqwest` is async. We use `tokio::runtime::Runtime`
in a blocking wrapper so the rest of Vaultic stays sync.

---

## Feature 2: Template Detection Improvements

### Config Changes

```toml
# config.toml additions
[vaultic]
version = "1.1.0"
format_version = 1          # NEW
default_cipher = "age"
default_env = "dev"
template = ".env.template"  # NEW: global default template path

[environments]
base = { file = "base.env" }
dev = { file = "dev.env", inherits = "base", template = "dev.env.template" }     # NEW
staging = { file = "staging.env", inherits = "base" }
prod = { file = "prod.env", inherits = "base", template = "prod.env.template" }  # NEW
```

### Template Resolution Order

For `vaultic check` (no --env):
1. Global `template` field in `[vaultic]` section
2. Auto-discovery in project root

For `vaultic check --env dev`:
1. `template` field in environment config (explicit)
2. `{env}.env.template` convention (e.g., `dev.env.template` in `.vaultic/`)
3. Global `template` field in `[vaultic]` section
4. Auto-discovery in project root

### Auto-Discovery Priority

When no template is configured, search project root in this order:
1. `.env.template` (current convention — highest priority)
2. `.env.example`
3. `.env.sample`
4. `env.template`

First match wins. If none found → descriptive error with suggestion.

### Per-Environment Templates Location

Templates for specific environments live inside `.vaultic/`:

```
.vaultic/
├── dev.env.template     # Template for dev environment
├── prod.env.template    # Template for prod environment
└── ...
```

These ARE committed to the repo (they contain only keys, no secret values).

### Modified Files

```
src/config/app_config.rs          # Add template field to VaulticSection and EnvEntry
src/cli/commands/check.rs         # Use new template resolution logic
src/core/services/check_service.rs # Accept dynamic template path
src/cli/commands/init.rs          # Update init to mention template discovery
```

### New Error Messages

```
✗ No template file found

  Vaultic searched for:
    ✗ .env.template
    ✗ .env.example
    ✗ .env.sample
    ✗ env.template

  Solutions:
    → Create a template: cp .env .env.template (then remove secret values)
    → Specify a path in .vaultic/config.toml:
      [vaultic]
      template = "path/to/your/template"
```

```
✗ Environment template not found for 'prod'

  Searched:
    ✗ .vaultic/prod.env.template (convention)
    ✗ .env.template (global fallback)

  Solutions:
    → Create .vaultic/prod.env.template with the expected variables
    → Or set template path in config.toml:
      [environments]
      prod = { file = "prod.env", inherits = "base", template = "prod.env.template" }
```

---

## Feature 3: Backward Compatibility

### Format Version

New field `format_version` in `config.toml`:

```toml
[vaultic]
format_version = 1   # Controls compatibility of config/data format
```

### Version Compatibility Rules

| Scenario | Action |
|----------|--------|
| `format_version` == current | Normal operation |
| `format_version` < current | Auto-migrate (with audit log entry) |
| `format_version` > current | Error: "Update Vaultic to work with this project" |
| `format_version` missing | Assume version 1 (backward compat with v1.1.0) |

### What Triggers a Format Version Bump

- Changes to `config.toml` schema
- Changes to `recipients.txt` format
- Changes to audit log format
- Changes to `.enc` file naming convention

### What Does NOT Trigger a Format Version Bump

- New CLI commands
- New optional fields in config.toml
- Bug fixes
- Performance improvements
- Changes to the encryption format itself (that's age's responsibility)

### Migration System

```rust
// core/services/migration_service.rs
pub fn migrate(config: &mut AppConfig, from: u32, to: u32) -> Result<()> {
    for version in from..to {
        match version {
            0 => migrate_v0_to_v1(config)?,  // hypothetical
            _ => {} // no migration needed
        }
    }
    Ok(())
}
```

Each migration:
1. Creates a backup of affected files
2. Applies the migration
3. Logs to audit: `action: "migrate", detail: "format_version 1 → 2"`
4. Updates `format_version` in config.toml

### Encrypted File Safety

The `.enc` files use standard age format (ASCII-armored). Vaultic does NOT add
any proprietary wrapper or metadata to encrypted files. This means:

- Files encrypted with Vaultic v1.0 will always be decryptable by any future version
- Files can even be decrypted with the standalone `age` CLI tool
- Format version only tracks Vaultic's OWN config format, not encryption format

---

## CLI Changes Summary

### New Command

```
vaultic update              # Check and apply update
```

### Modified Commands

```
vaultic check               # Now with auto-discovery and per-env template support
vaultic check --env dev     # Uses dev-specific template if available
vaultic init                # Generates config.toml with format_version field
```

### New Global Behavior

```
# Every command (except --quiet):
# After output, shows update banner if new version available (max 1x/24h)
```

---

## Dependencies Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| reqwest | 0.12 | HTTP client for GitHub API and downloads |
| semver | 1 | Semantic version comparison |
| minisign-verify | 0.2 | Verify minisign signatures (read-only) |
| self_replace | 1 | Atomic binary self-replacement |
| tokio | 1 | Async runtime for reqwest (minimal features) |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| GitHub API rate limit (60 req/h unauthenticated) | Cache check result, only 1x/24h |
| Binary replacement fails mid-write | `self_replace` uses atomic operations |
| Minisign secret key leaked | Rotate key, publish new public key in next release |
| User behind corporate proxy | `reqwest` respects HTTP_PROXY/HTTPS_PROXY env vars |
| Update introduces breaking format change | `format_version` + migration system protects users |
| Template field breaks existing config.toml | `template` fields are optional with sane defaults |
