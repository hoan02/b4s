# Changelog

## 0.1.5 (beta) — 2026-10-13

Published as a GitHub pre-release: installed 0.1.4 copies do not auto-update
to it. The changes below are traced from APK source and have not been
accepted on hardware yet; please report results for BP1 Pro ANC, BP1 Ultra
LDAC and restore defaults.

- Re-audit APK 2.17.0.1 with REA (headless JADX) and fix the ported protocol:
  LDAC on/off was inverted (`BA75 01` now enables, `AA74 01` is enabled),
  restore defaults completes on `AA37 01`, and noise writes are confirmed from
  the `AA33` readback because `AA34` is only a write status.
- Stop the 789C decoder from inventing frames out of `0xAA` payload or length
  bytes and keep every sub-frame in device order.
- Apply per-model APK limits: bass 1-3 on BP1 Ultra, EP10 Ultra and M4s; noise
  environments per model, including the self-adaptive mode (104); one
  transparency mode on the BP1 Pro family; custom EQ gain +/-6 dB.
- Remove controls the APK does not offer: BP1 Pro adaptive L/R and EP10 Pro EQ
  presets copied from BP1 Pro. Drop the short `m3s`/`m4s`/`ms1` aliases.
- Find earbuds can target the left, right or both buds and is available on all
  seven control profiles. Wind noise (BP1 Ultra family) and LDAC (BP1 Pro
  family) appear only after the earbuds answer their state query.
- Show the earbuds' own rejection reason (dual connection, in a call, LDAC on,
  wear both buds...) instead of a timeout, and block legacy gesture writes when
  firmware reports the V2 gesture contract.

## 0.1.4 — 2026-10-08

- Unify the control UI: grouped list rows, icons, battery bars and select
  controls across the home, gesture and sound screens.
- Replace the README screenshots with a narrated demo video and GIF preview, and
  call for contributors and hardware testers for the experimental profiles.
- Add experimental profiles for Bass EP10 Ultra, Bowie M4s and MS1 (BP1 Ultra
  adapter) and Bowie M3s and Bass EP10 Pro (BP1 Pro adapter). They are derived
  from exact-name APK evidence and are not hardware verified; M3s has no EQ.
- Add an Experimental-only wind noise reduction toggle (`BA51`/`BA52`) for BP1 Pro,
  Bowie M3s and EP10 Pro, confirmed by `BA51` readback.
- Correct the BA5E protocol comment and record the APK cross-check of AA37,
  LDAC polarity and `#InitState` in the parity audit.

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
