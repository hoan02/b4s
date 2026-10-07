# Changelog

## 0.1.3 — 2026-10-07

- Embed 124 headphone profiles: retain BP1 Pro and Ultra controls and add 122
  recognition-only profiles. Other models do not gain unverified controls.
- Standardize model IDs/filenames and migrate remembered device IDs and custom EQ.
- Separate APK model identity, product presentation, images and color metadata.
- Cache product art locally with a 64 MiB budget, smaller scan thumbnails and
  remembered-device image warmup. Cached images work offline.
- Require schema-3 feature evidence and explicit sound/EQ/gesture constraints.
- Align Tauri API/CLI 2.12.1 and updater 2.13.1 across Rust and frontend.

**Upgrade from 0.1.1/0.1.2:** install 0.1.3 manually once. Those versions embed
the previous updater public key and cannot verify this release's new signature.
Later releases will retain the new signing key.

APK feature parity is incomplete. Gesture V2, Classic control transport and
Ultra EQ/SoundFit remain outside the implemented support scope. No new model
hardware acceptance or native image performance measurement is claimed.

Validation: 49 frontend tests, 166 Rust tests (one hardware test ignored), seven
catalog parser tests and 15 extraction tests passed. Frozen dependency install,
frontend build/type-check, locale checks, locked cargo check and updater signing
key compatibility checks passed locally.
