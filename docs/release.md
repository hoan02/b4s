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
| `src-tauri/tauri.conf.json` | `version` |
| `src-tauri/Cargo.toml` | `version` |

```bash
npm run version:bump          # patch
npm run version:bump -- minor
npm run version:bump -- major
npm run version:bump -- 1.4.0
```

## One-time: signing key

1. Generate (if needed): `npx tauri signer generate -w .tauri/b4s.key`  
2. GitHub → **Settings → Secrets → Actions**  
3. `TAURI_SIGNING_PRIVATE_KEY` = contents of `.tauri/b4s.key`  
4. Put public key in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`  

`.tauri/` private keys are **gitignored**.

## Publish (recommended)

Working tree must be clean. Script bumps version files, commits, tags `vX.Y.Z`, and pushes — that triggers the **Release** workflow.

```bash
npm run release                 # patch  0.1.0 → 0.1.1
npm run release -- minor        #        0.1.0 → 0.2.0
npm run release -- major        #        0.1.0 → 1.0.0
npm run release -- 0.2.0        # exact version

npm run release -- patch --dry-run   # preview only
npm run release -- --no-bump         # tag current version, no bump
npm run release -- patch --no-push   # commit + tag local only
```

### Manual (equivalent)

```bash
npm run version:bump
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: release v$(node -p "require('./package.json').version")"
git tag "v$(node -p "require('./package.json').version")"
git push origin main --tags
```

Updater endpoint:

`https://github.com/hoan02/b4s/releases/latest/download/latest.json`
