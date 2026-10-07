# Repository Guidelines

## Project Structure & Module Organization

B4S is a SolidJS/Vite frontend packaged as a cross-platform Tauri desktop app.

- `src/` contains SolidJS entry points, components, frontend state, BLE bridges, and SCSS styles.
- `src-tauri/src/` contains Rust commands and application logic; BLE integration is in `ble/`, device registration in `device/`, and protocol framing/adapters in `protocol/`.
- `src-tauri/catalog/` stores model capability profiles, while `docs/` documents reverse-engineering and model additions.
- `public/` and `assets/` contain static images and other UI assets; `scripts/` contains version and release helpers.
- `.github/workflows/` defines CI and release automation. Do not commit generated `target/` or dependency artifacts.

## Build, Test, and Development Commands

Install dependencies with `bun install --frozen-lockfile` (or `bun install` when updating `bun.lock`). CI uses Bun 1.4.0. Use:

```bash
bun run dev                 # Start the Vite frontend
bun run tauri:dev           # Run the desktop app with the Rust backend
bun run build               # Type-check and build the frontend
bun x --bun tsc --noEmit    # Frontend type-check only
cargo check --manifest-path src-tauri/Cargo.toml  # Check Rust code
bun run tauri:build         # Build platform installers
```

Test on hardware with Bluetooth enabled and a supported, pairable earbud nearby. CI runs frontend type-check/build, `cargo check`, and a Tauri build on Windows and Ubuntu.

## Coding Style & Naming Conventions

Use two spaces in TypeScript/TSX, Rust’s standard `rustfmt` style, and clear types at frontend/backend boundaries. Name Solid components in PascalCase (`DeviceHeader.tsx`), utilities in camelCase, Rust modules/files in snake_case, and model catalog files with lowercase kebab-case (`bass-bp1-pro.json`). Keep protocol changes isolated to the relevant adapter and document newly verified behavior.

## Testing Guidelines

Frontend tests use Bun's test runner (`bun run test:session`, `bun run test:preferences`, `bun run test:reconnect`, `bun run test:logging`, and `bun run test:storage`). Before submitting changes, run the type-check, frontend build, and `cargo check`; exercise affected BLE flows manually when hardware is available. Add focused Rust or frontend tests alongside new logic if introducing behavior that can be tested without hardware.

## Commit & Pull Request Guidelines

Follow the existing concise, imperative commit style with a conventional prefix where useful, such as `feat:`, `ci:`, `docs:`, or `fix:` (example: `feat: add BP1 profile`). Keep commits focused. Pull requests should explain the user-visible or protocol impact, list validation commands, link related issues, and include screenshots or recordings for UI changes. Call out hardware, firmware, platform, and manual-test limitations.

## Security & Configuration Tips

Never commit updater private keys, credentials, proprietary APK/decompiled source, firmware, or personal device data. Release signing uses `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in CI secrets. Treat unverified device profiles and protocol commands as experimental and test them on non-critical hardware.
