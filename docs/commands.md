# Command Reference

> English | **[Español](commands.es.md)**

Complete reference for all Vaultic CLI commands with examples and explanations.

## Table of Contents

- [Global Flags](#global-flags)
- [Environment Variables](#environment-variables)
- [init](#vaultic-init)
- [encrypt](#vaultic-encrypt)
- [decrypt](#vaultic-decrypt)
- [check](#vaultic-check)
- [template sync](#vaultic-template-sync)
- [validate](#vaultic-validate)
- [diff](#vaultic-diff)
- [resolve](#vaultic-resolve)
- [keys setup](#vaultic-keys-setup)
- [keys add](#vaultic-keys-add)
- [keys list](#vaultic-keys-list)
- [keys remove](#vaultic-keys-remove)
- [log](#vaultic-log)
- [status](#vaultic-status)
- [hook install / uninstall](#vaultic-hook)
- [ci export](#vaultic-ci-export)
- [Common Workflows](#common-workflows)

---

## Global Flags

These flags work with any command:

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--cipher <age\|gpg>` | — | `age` | Encryption backend |
| `--env <name>` | — | `dev` | Target environment (repeatable for diff) |
| `--config <path>` | — | `.vaultic/` | Custom vaultic directory path |
| `--verbose` | `-v` | off | Show detailed output |
| `--quiet` | `-q` | off | Suppress all output except errors |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VAULTIC_AGE_KEY` | When set, Vaultic uses its value as the age private key instead of reading from `~/.config/age/keys.txt`. Essential for CI/CD pipelines where key files are not available. Empty values are rejected with a clear error. |

**Example usage in CI:**

```bash
# GitHub Actions
- run: vaultic decrypt --env prod --stdout > .env
  env:
    VAULTIC_AGE_KEY: ${{ secrets.VAULTIC_AGE_KEY }}

# Generic CI
export VAULTIC_AGE_KEY="AGE-SECRET-KEY-1QFWZ..."
vaultic ci export --env prod --format generic
```

Supported in: `decrypt`, `resolve`, `ci export`, `encrypt --all`, and all commands that decrypt in memory.

---

## `vaultic init`

Initialize Vaultic in a new project. Creates the `.vaultic/` directory with configuration files and optionally generates your encryption key.

```
vaultic init
```

**What it does:**

1. Creates `.vaultic/` directory
2. Generates `config.toml` with default environments (base, dev, staging, prod)
3. Creates empty `recipients.txt`
4. Creates `.env.template`
5. Adds `.env` to `.gitignore`
6. Searches for existing age/GPG keys on your system
7. If no key found, asks if you want to generate one
8. Records the operation in the audit log

**Interactive key detection:**

- If you answer **Y**: generates an age key at `~/.config/age/keys.txt` and adds your public key to `recipients.txt`
- If you answer **N**: skips key generation — you can run `vaultic keys setup` later

**Example:**

```
$ vaultic init

🔐 Vaultic — Initializing project
  ✓ Created .vaultic/
  ✓ Generated config.toml with defaults
  ✓ Created .env.template
  ✓ Added .env to .gitignore

🔑 Key configuration
  No age key found. Generate one now? [Y/n]: Y

  ✓ Private key saved to: ~/.config/age/keys.txt
  ✓ Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
  ✓ Public key added to .vaultic/recipients.txt
  ✓ Project ready.
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "already initialized" | `.vaultic/` already exists | Project is already set up — no action needed |

---

## `vaultic encrypt`

Encrypt a plaintext file so it can be safely committed to Git.

```
vaultic encrypt [FILE] [--env <name>] [--all] [--cipher <age|gpg>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `FILE` | `.env` | Source file to encrypt |
| `--env <name>` | `dev` | Environment label for the encrypted file |
| `--all` | off | Re-encrypt all environments (ignores FILE and --env) |

**What it does:**

1. Reads your plaintext file (e.g. `.env`)
2. Encrypts it with the public keys of all recipients in `recipients.txt`
3. Saves the result as `.vaultic/{env}.env.enc`
4. The original file is NOT modified or deleted

**The `--env` flag** is a label that names the encrypted file. Different environments have different secrets:

```bash
vaultic encrypt .env --env dev       # → .vaultic/dev.env.enc
vaultic encrypt .env --env staging   # → .vaultic/staging.env.enc
vaultic encrypt .env --env prod      # → .vaultic/prod.env.enc
```

**The `--all` flag** re-encrypts every environment defined in `config.toml`. This is essential after adding or removing a team member:

```bash
# After adding a new recipient
vaultic keys add age1x9ynm5k...
vaultic encrypt --all    # Re-encrypts all envs so the new member can decrypt
```

How `--all` works: it decrypts each `.enc` file in memory (no plaintext on disk) and re-encrypts with the current recipient list.

**Example:**

```
$ vaultic encrypt .env --env dev

  Source: .env
  ⏳ Encrypting dev with age for 3 recipient(s)...
  ✓ Encrypted with age for 3 recipient(s)
  ✓ Saved to .vaultic/dev.env.enc

  Commit .vaultic/dev.env.enc to the repo.
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "not initialized" | `.vaultic/` missing | Run `vaultic init` first |
| "No recipients" | `recipients.txt` is empty | Run `vaultic keys add <key>` |
| "Unknown cipher" | Invalid `--cipher` value | Use `age` or `gpg` |

---

## `vaultic decrypt`

Decrypt an encrypted file to restore your local `.env`.

```
vaultic decrypt [FILE] [--env <name>] [--key <path>] [-o <path>] [--stdout] [--cipher <age|gpg>]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `FILE` | — | `.vaultic/{env}.env.enc` | Encrypted file to decrypt |
| `--env <name>` | — | `dev` | Environment to decrypt |
| `--key <path>` | — | `~/.config/age/keys.txt` | Path to your private key |
| `--output <path>` | `-o` | `.env` | Where to write the decrypted file |
| `--stdout` | — | off | Write decrypted content to stdout (mutually exclusive with `-o`) |

**What it does:**

1. Reads the encrypted file (`.vaultic/dev.env.enc`)
2. Decrypts it using your private key
3. Writes the plaintext to the output path (default: `.env`)
4. Shows how many variables were decrypted

**The `--key` flag** lets you use a private key from a custom location instead of the default:

```bash
vaultic decrypt --env dev --key /path/to/my-key.txt
```

**The `-o` flag** lets you write the decrypted output to a custom path:

```bash
vaultic decrypt --env dev -o backend/.env     # Decrypt to a subdirectory
vaultic decrypt --env prod -o deploy/.env     # Decrypt prod to deploy folder
```

**The `--stdout` flag** writes the decrypted content directly to stdout with no UI messages. This enables piping to other tools:

```bash
vaultic decrypt --env dev --stdout | docker run --env-file - myapp
vaultic decrypt --env prod --stdout > /tmp/prod.env
```

`--stdout` and `-o` are mutually exclusive — use one or the other.

**Example:**

```
$ vaultic decrypt --env dev

  Source: .vaultic/dev.env.enc
  ⏳ Decrypting dev with age...
  ✓ Decrypted .vaultic/dev.env.enc
  ✓ Generated .env with 23 variables

  Run 'vaultic check' to verify no variables are missing.
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "not found" | Encrypted file missing | Check env name with `vaultic status` or `git pull` |
| "No private key found" | Key file missing | Run `vaultic keys setup` or use `--key <path>` |
| "no matching key found" | Your key isn't in the recipient list | Ask an admin to run `vaultic keys add <your_key>` |

---

## `vaultic check`

Compare your local `.env` against `.env.template` to detect missing or extra variables.

```
vaultic check
```

No flags — it always compares `.env` vs `.env.template` in the project root.

**What it reports:**

- **Missing variables**: exist in template but not in your `.env`
- **Extra variables**: exist in your `.env` but not in template
- **Empty values**: variables with no value assigned

**Example:**

```
$ vaultic check

  🔍 vaultic check
  ⚠ Missing variables (2):
      • REDIS_CLUSTER_URL
      • FEATURE_FLAG_V2

  ⚠ Extra variables not in template (1):
      • OLD_API_KEY

  21/23 variables present, 2 issue(s) found
```

If everything is in sync:

```
$ vaultic check

  ✓ 23/23 variables present — all good
```

---

## `vaultic template sync`

Auto-generate `.env.template` from all encrypted environments. Decrypts each environment in memory, collects the union of all keys, strips all values, and writes the result.

```bash
# Sync .env.template from all encrypted environments
vaultic template sync

# Write to a custom path
vaultic template sync -o custom.template
```

**What it does:**
1. Decrypts each `.env.enc` file in memory (requires your private key)
2. Collects every key from every environment (union)
3. Strips all values (empty strings)
4. Writes the result to `.env.template` (or custom path)

This keeps your template always in sync with the actual secrets — no manual maintenance needed. The output file is safe to commit.

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output <path>` | Write to a custom path instead of `.env.template` |

---

## `vaultic validate`

Validate your local `.env` against format rules defined in `.vaultic/config.toml`.

```bash
# Validate .env
vaultic validate

# Validate a specific file
vaultic validate -f prod.env
```

**Example output:**
```
🔍 vaultic validate
  File: .env
  Rules: 5 defined

  ✗ STRIPE_KEY — does not match pattern "^sk_live_.*"
  ✓ API_KEY — ok
  ✓ DATABASE_URL — ok
  ✓ DEBUG — ok
  ✓ PORT — ok

  4/5 rules passed
```

**Configuring rules** in `.vaultic/config.toml`:

```toml
[validation]
DATABASE_URL = { type = "url", required = true }
PORT = { type = "integer", min = 1024, max = 65535 }
API_KEY = { type = "string", min_length = 32 }
DEBUG = { type = "boolean" }
STRIPE_KEY = { pattern = "^sk_live_.*" }
```

**Supported rule fields:**

| Field | Description | Example |
|-------|-------------|---------|
| `type` | Value type: `url`, `integer`, `boolean`, `string` | `type = "url"` |
| `required` | Key must be present and non-empty | `required = true` |
| `min` / `max` | Numeric bounds (integer type) | `min = 1024, max = 65535` |
| `min_length` / `max_length` | String length bounds | `min_length = 32` |
| `pattern` | Regex pattern the value must match | `pattern = "^sk_live_.*"` |

All fields are optional and combinable. If a key is not required and is absent, it is silently skipped.

**CI-friendly:** exits with code 1 on failure, making it suitable for CI pipelines.

**Options:**

| Flag | Description |
|------|-------------|
| `-f, --file <path>` | File to validate (default: `.env`) |

---

## `vaultic diff`

Compare two secret files or two resolved environments side by side.

**File mode** — compare two plaintext files:

```
vaultic diff <file1> <file2>
```

**Environment mode** — compare two resolved environments (decrypts and applies inheritance):

```
vaultic diff --env <name1> --env <name2>
```

**What it shows:**

| Color | Meaning |
|-------|---------|
| Green | Added — exists in the second but not the first |
| Red | Removed — exists in the first but not the second |
| Yellow | Modified — same key, different values |

**Example:**

```
$ vaultic diff --env dev --env prod

  Comparing environments: dev vs prod

  Variable            │ dev           │ prod
  ────────────────────┼───────────────┼──────────────
  DATABASE_URL        │ localhost     │ rds.aws.com
  DEBUG               │ true          │ ✗ (missing)
  REDIS_CLUSTER       │ ✗ (missing)   │ redis.prod

  ✓ 1 added, 1 removed, 1 modified
```

This is useful to catch configuration drift between environments — for example, a variable that exists in dev but was forgotten in prod.

---

## `vaultic resolve`

Generate a final `.env` file by merging environment layers (base + overlay).

```
vaultic resolve --env <name> [-o <path>] [--stdout] [--cipher <age|gpg>]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--env <name>` | — | from config | Environment to resolve |
| `--output <path>` | `-o` | `.env` | Where to write the resolved file |
| `--stdout` | — | off | Write resolved content to stdout (mutually exclusive with `-o`) |

**The `--stdout` flag** works the same as in `decrypt` — raw env content to stdout, no UI messages:

```bash
vaultic resolve --env prod --stdout | docker run --env-file - myapp
```

**How inheritance works:**

Your `config.toml` defines inheritance chains:

```toml
[environments]
base = "base.env"
dev = { file = "dev.env", inherits = "base" }
staging = { file = "staging.env", inherits = "base" }
prod = { file = "prod.env", inherits = "base" }
```

When you run `vaultic resolve --env prod`:

1. Decrypts `base.env.enc` → gets base variables
2. Decrypts `prod.env.enc` → gets prod variables
3. Merges: prod overrides base where keys conflict
4. Writes the final result to `.env`

All decryption happens in memory — no intermediate plaintext files on disk.

**Example:**

```
$ vaultic resolve --env prod

  Resolving environment: prod
  ✓ Inheritance chain: base → prod
  ✓ Resolved 42 variables from 2 layer(s)
  ✓ Written to .env

$ vaultic resolve --env staging -o deploy/.env
  # Resolves staging and writes to deploy/.env
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "Environment not found" | Name not in `config.toml` | Check spelling or add it to config |
| "Circular inheritance" | e.g. dev → staging → dev | Fix the chain in `config.toml` |

---

## `vaultic keys setup`

Interactive key generation or import for new users.

```
vaultic keys setup
```

**Presents an interactive menu:**

1. **Generate new age key** (recommended) — creates a keypair at `~/.config/age/keys.txt`
2. **Import existing age key from file** — copies your key to the standard location
3. **Use existing GPG key** — if GPG is available on your system

After setup, it displays your public key and instructions for the project admin:

```
$ vaultic keys setup

  ✓ Key generated
  ✓ Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p

  📋 Next step:
     Send your PUBLIC key to the project admin.
     The admin will run: vaultic keys add age1ql3z7hjy...ac8p
     Then you can decrypt with: vaultic decrypt --env dev
```

**Is it safe to share the public key?** Yes. The public key can only encrypt data for you — it cannot decrypt anything. Think of it as an open padlock: anyone can lock it, but only you have the key to open it.

---

## `vaultic keys add`

Add a recipient's public key to the authorized list.

```
vaultic keys add <KEY>
```

**Accepted key formats:**

| Format | Example |
|--------|---------|
| age public key | `age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p` |
| GPG email | `user@example.com` |
| GPG fingerprint | `ABCDEF1234567890...` |

**After adding a key, you must re-encrypt** so the new member can decrypt:

```bash
vaultic keys add age1x9ynm5k...
vaultic encrypt --all
git add .vaultic/ && git commit -m "chore: add new team member"
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "already exists" | Key already in `recipients.txt` | No action needed |
| "Invalid age public key" | Malformed key | Verify key starts with `age1` |

---

## `vaultic keys list`

List all authorized recipients.

```
vaultic keys list
```

**Example:**

```
$ vaultic keys list

  📋 Authorized recipients (3)
  • age1ql3z7hjy...ac8p
  • age1x9ynm5k...7f2p
  • age1htr8gqn...9d3k  # team-lead
```

Labels after `#` are optional comments added to `recipients.txt`.

---

## `vaultic keys remove`

Remove a recipient from the authorized list.

```
vaultic keys remove <KEY>
```

**After removing a key, you must re-encrypt** to revoke access:

```bash
vaultic keys remove age1x9ynm5k...
vaultic encrypt --all
git add .vaultic/ && git commit -m "chore: remove departed member"
```

Previously encrypted files in Git history remain decryptable by the removed key — rotate sensitive secrets after removing a member.

---

## `vaultic log`

Show the audit log of all operations.

```
vaultic log [--author <name>] [--since <date>] [--last <n>]
```

| Option | Format | Description |
|--------|--------|-------------|
| `--author <name>` | free text | Filter by Git author name |
| `--since <date>` | `YYYY-MM-DD` | Show entries from this date onward |
| `--last <n>` | integer | Show only the last N entries |

**Example:**

```
$ vaultic log --last 5

  Date/Time            │ Author   │ Action   │ Detail
  ─────────────────────┼──────────┼──────────┼─────────────────
  2026-02-23 14:30:00  │ Cristo   │ encrypt  │ dev.env.enc
  2026-02-23 10:15:00  │ María    │ decrypt  │ prod → .env
  2026-02-22 18:45:00  │ Cristo   │ check    │ 23/23 present
  2026-02-22 16:20:00  │ Alex     │ key add  │ age1x9y...
  2026-02-22 09:00:00  │ Cristo   │ init     │ —

  Showing 5 entries

$ vaultic log --author "Cristo" --since 2026-02-22
  # Shows only Cristo's entries from Feb 22 onward
```

The audit log never contains secret values — only operation metadata (action, files, timestamps).

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid date format" | `--since` value not `YYYY-MM-DD` | Use ISO 8601 format |

---

## `vaultic status`

Show a complete overview of the project configuration and state.

```
vaultic status
```

**Example:**

```
$ vaultic status

  🔐 Vaultic v1.1.0
  Cipher: age
  Config: .vaultic/config.toml

  Recipients (3):
  • age1ql3z7hjy...ac8p
  • age1x9ynm5k...7f2p
  • age1htr8gqn...9d3k

  Encrypted environments:
  ✓ base.env.enc
  ✓ dev.env.enc
  ✓ staging.env.enc
  ✓ prod.env.enc
  ✗ testing (not encrypted)
```

---

## `vaultic hook`

Install or uninstall a Git pre-commit hook that blocks accidental commits of plaintext `.env` files.

**Install:**

```
vaultic hook install
```

**Uninstall:**

```
vaultic hook uninstall
```

**What the hook does:**

When you run `git commit`, the hook scans staged files. If it finds a plaintext `.env` file, it blocks the commit:

```
🚨 Vaultic pre-commit hook

  Plaintext .env file detected in staged files!
  Encrypt first: vaultic encrypt
  Or bypass (not recommended): git commit --no-verify
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "Not a git repository" | No `.git/` directory | Run `git init` first |
| "not installed by Vaultic" | Existing hook from another tool | Remove it manually or keep your existing hook |

---

## `vaultic ci export`

Export secrets in CI-specific formats. Designed for use in CI/CD pipelines.

```
vaultic ci export --env <name> [--format <github|gitlab|generic>] [--mask] [--cipher <age|gpg>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--env <name>` | `dev` | Environment to export |
| `--format <format>` | `generic` | Output format: `github`, `gitlab`, or `generic` |
| `--mask` | off | Emit `::add-mask::` commands for GitHub Actions (requires `--format github`) |

**Output formats:**

| Format | Output per variable | Use case |
|--------|-------------------|----------|
| `generic` | `KEY=value` | General purpose, piping |
| `github` | `echo "KEY=value" >> "$GITHUB_ENV"` | GitHub Actions workflows |
| `gitlab` | `export KEY="value"` | GitLab CI scripts |

**The `--mask` flag** adds `::add-mask::value` lines before each variable when using `--format github`, preventing secret values from appearing in GitHub Actions logs.

**Examples:**

```bash
# GitHub Actions workflow
- run: |
    eval "$(vaultic ci export --env prod --format github --mask)"
  env:
    VAULTIC_AGE_KEY: ${{ secrets.VAULTIC_AGE_KEY }}

# GitLab CI
script:
  - eval "$(vaultic ci export --env prod --format gitlab)"

# Generic — pipe to file
vaultic ci export --env dev --format generic > .env
```

**Errors:**

| Error | Cause | Solution |
|-------|-------|----------|
| "Unsupported format" | Invalid `--format` value | Use `github`, `gitlab`, or `generic` |
| "--mask requires --format github" | `--mask` used without GitHub format | Add `--format github` |

---

## Common Workflows

### First time setup (new project)

```bash
vaultic init                           # Create .vaultic/ and generate key
echo "DATABASE_URL=localhost" > .env   # Create your .env
vaultic encrypt --env dev              # Encrypt it
git add .vaultic/ .env.template        # Commit encrypted file + template
git push
```

### Joining an existing project

```bash
git clone <repo> && cd <project>
vaultic keys setup                     # Generate your key
# Send your PUBLIC key to the admin
# Admin runs: vaultic keys add <your_key> && vaultic encrypt --all
vaultic decrypt --env dev              # Decrypt your local .env
vaultic check                          # Verify nothing is missing
```

### After changing secrets

```bash
# Edit .env with new values
vaultic encrypt --env dev              # Re-encrypt
git add .vaultic/dev.env.enc
git commit -m "chore: update dev secrets"
```

### Adding a team member

```bash
vaultic keys add <their_public_key>    # Add their key
vaultic encrypt --all                  # Re-encrypt all envs for new member
git add .vaultic/
git commit -m "chore: add new team member"
```

### Removing a team member

```bash
vaultic keys remove <their_public_key> # Remove their key
vaultic encrypt --all                  # Re-encrypt without them
# Rotate sensitive secrets (API keys, passwords)
git add .vaultic/
git commit -m "chore: revoke departed member access"
```

### Comparing environments before deploy

```bash
vaultic diff --env staging --env prod  # See what differs
vaultic resolve --env prod -o .env     # Get the resolved prod config
```

### Using Vaultic in CI/CD

```bash
# Set the private key as an environment variable (no key file needed)
export VAULTIC_AGE_KEY="AGE-SECRET-KEY-1QFWZ..."

# Export secrets for GitHub Actions
vaultic ci export --env prod --format github --mask

# Or pipe decrypted output directly to Docker
vaultic decrypt --env dev --stdout | docker run --env-file - myapp

# Or resolve and write to a file for deployment
vaultic resolve --env prod --stdout > deploy/.env
```
