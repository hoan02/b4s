# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

B4S is an unofficial desktop companion for Baseus-style BLE earbuds: a SolidJS/Vite frontend packaged with Tauri 2, backed by Rust. See `AGENTS.md` for style, commit and PR conventions, and `docs/architecture.md` for the full architecture write-up.

## Commands

Bun 1.4.0 is the package manager.

```bash
bun install --frozen-lockfile
bun run tauri:dev                                  # full app (frontend + Rust)
bun run dev                                        # Vite frontend only
bun run build                                      # tsc + vite build
bun x --bun tsc --noEmit                           # type-check only
cargo check --manifest-path src-tauri/Cargo.toml   # Rust check
cargo test --manifest-path src-tauri/Cargo.toml    # Rust tests
bun run check:i18n                                 # locale key consistency (runs in CI)
```

Frontend tests are Bun tests in `scripts/*.test.mjs`, grouped by script: `test:models`, `test:storage`, `test:logging`, `test:session`, `test:preferences`, `test:reconnect`. Run a single file with `bun test scripts/reconnect.test.mjs`.

Release helpers: `bun run version:bump`, `bun run release` (see `docs/release.md`).

## Architecture

Frontend (`src/`) calls Rust (`src-tauri/src/`) through typed Tauri commands/events. Separate owners for each concern:

- **Frontend**: `src/App.tsx` shell; `src/stores/deviceSession.ts` orders session state; `src/features/<feature>/controller.ts` run feature operations; `src/lib/ble.ts` and `src/bridge/deviceSnapshot.ts` are the typed adapters that validate backend DTOs.
- **Model catalog**: `src-tauri/catalog/models/*.json` (reviewed profiles) and `catalog/baseus-public.json` (passive identity/display only). `build.rs` embeds the profiles; `src-tauri/src/catalog/` parses and validates them (duplicate IDs, unregistered families, bad capabilities, inconsistent EQ fail validation).
- **Device layer** (`src-tauri/src/device/`): `registry.rs` does exact name/alias identity resolution; `capability.rs`/`initialization.rs` authorize operations and plan startup queries; `snapshot.rs` holds confirmed state.
- **BLE** (`src-tauri/src/ble/`): discovery, connection, commands and GATT transport are separate modules; shared mutable state is owned by `runtime.rs`.
- **Protocol** (`src-tauri/src/protocol/`): `router.rs` dispatches encode/decode by the selected profile's family; codecs live in `families/` (`bp1.rs`, `bp1_ultra.rs`).

## Invariants to preserve

- **Reviewed profiles are the only source of control permissions.** Public catalog metadata is scan-only. Never infer capabilities, framing or edition from name substrings or a similar model; unresolved models stay scan-only.
- **No protocol fallbacks.** Unknown families must not inherit BP1 decoding. A new family needs its own codec, registration in catalog validation, and test vectors (commands, replies, malformed frames, readback). Transport uses only the selected OS device entry and the profile's declared UUIDs/handshake/framing; no generic characteristic probing, same-name sibling switching, or re-keying by address. Scan keeps distinct OS entry IDs even with the same Bluetooth address.
- **Confirmed state comes from device snapshots, not write completion.** Snapshots carry session ID + revision to reject stale updates after reconnect. Snapshot schema is v2 (ANC mode/param observations are same-session only).
- **Contract versioning**: scan, connection and link-health DTOs carry a version; frontend adapters reject unsupported versions. Change both sides in the same commit and bump the version when the payload shape changes.
- **Stored model IDs** change only via a deliberate, versioned one-time migration (`src/lib/modelIdMigration.ts`), never for live routing.
- Hardware verification is recorded per feature/model/firmware/transport/platform; synthetic replay does not promote support to "verified". Track evidence in `docs/headphone-desktop-progress.md` and `docs/model-support-matrix.md`.

## Localization

English is default and fallback; five locales live in `src/locales/<locale>/translation.json` and are registered in `src/lib/i18n.ts`. All visible strings go through i18n; run `bun run check:i18n` after changes (see `docs/translations.md`).
