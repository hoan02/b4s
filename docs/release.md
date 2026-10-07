# Release & auto-update (B4S)

Repo: [hoan02/b4s](https://github.com/hoan02/b4s)

B4S ships via **GitHub Releases** and can **self-update** (Tauri updater).

## Why Actions looked empty after first push

| Workflow | When it runs |
|----------|----------------|
| **CI** | Every push / PR to `main` (build check, no Release) |
| **Release** | Only on **tag** `v*` (e.g. `v0.1.0`) or **Actions → Release → Run workflow** |

Pushing code to `main` alone does **not** create installers or a Release.

## Version files (must stay in sync)

| File | Field |
|------|--------|
| `package.json` | `version` |
| `bun.lock` | Dependency lockfile (no app version field) |
| `src-tauri/tauri.conf.json` | `version` |
| `src-tauri/Cargo.toml` | `version` |
| `src-tauri/Cargo.lock` | `b4s` package version |

```bash
bun run version:bump          # patch
bun run version:bump minor
bun run version:bump major
bun run version:bump 1.4.0
```

## One-time: signing key

1. Generate (if needed): `bun x --bun tauri signer generate -w .tauri/b4s.key`
2. GitHub → **Settings → Secrets → Actions**  
3. `TAURI_SIGNING_PRIVATE_KEY` = contents of `.tauri/b4s.key`  
4. Put public key in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`  

`.tauri/` private keys are **gitignored**.

### Updater key recovery (2026-10-07)

The updater key was replaced after the original key was lost. The current
private key is stored locally at `.tauri/updater-2026-10-07.key`, with its public
key in the adjacent `.key.pub` file and `plugins.updater.pubkey`. The matching
GitHub Actions secrets have been configured with an empty key password.
Back up the private key in a secure password manager or encrypted offline storage;
GitHub Secrets cannot be used to retrieve it later. Do not generate a new key
for each release or commit the private key.

Release CI runs `bun scripts/check-updater-key.mjs` before building to reject
signatures whose key ID differs from the configured public key.
For local builds in PowerShell:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content .tauri/updater-2026-10-07.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
bun scripts/check-updater-key.mjs
bun run tauri:build
```

Versions 0.1.1 and 0.1.2 contain the previous public key. Users must download
and install the 0.1.3 release containing the new public key manually once.
Subsequent releases must keep this same key pair for automatic updates to work.
Do not replace the existing 0.1.2 release; publish a new version for migration.

## Publish (recommended)

`main` is protected: prepare the version bump and changes on a branch, open a
pull request, and wait for the required Windows/Ubuntu checks before merging.
After merging, update the local `main`, tag the merge commit and push that tag.
Do not tag an unmerged feature commit or bypass branch protection. The helper
below is suitable for a clean, already merged version with `--no-bump`; a direct
version-bump push to protected `main` will be rejected.

Working tree must be clean. Script bumps version files, commits, tags `vX.Y.Z`, and pushes — that triggers the **Release** workflow.

```bash
bun run release                 # patch  0.1.0 → 0.1.1
bun run release minor        #        0.1.0 → 0.2.0
bun run release major        #        0.1.0 → 1.0.0
bun run release 0.2.0        # exact version

bun run release patch --dry-run   # preview only
bun run release --no-bump         # tag current version, no bump
bun run release patch --no-push   # commit + tag local only
```

### Manual (equivalent)

```bash
bun run version:bump
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: release v$(bun -p "require('./package.json').version")"
git tag "v$(bun -p "require('./package.json').version")"
git push origin main --tags
```

Updater endpoint:

`https://github.com/hoan02/b4s/releases/latest/download/latest.json`

## 0.1.3 release checklist

See [CHANGELOG](../CHANGELOG.md) for public release notes. Check frozen Bun install, all frontend tests, i18n, Python catalog/extraction tests, frontend build, serial Rust tests and locked cargo check. The Release workflow verifies signing-key compatibility before each platform build. Verify Windows/macOS/Linux installers, detached signatures and `latest.json` after publication. APKs, extraction outputs and private signing keys stay ignored.
