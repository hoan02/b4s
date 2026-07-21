# Model Profile Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the companion app so the connection, capability selection, noise UI, and BLE command routing are model/firmware driven like Baseus APK 2.14.1 instead of BP1-driven.

**Architecture:** Keep BLE transport and protocol framing separate from a `DeviceProfile` resolved from catalog model, firmware, and observed protocol. The profile owns mode IDs, adaptive environments, custom ANC range, and protocol family; UI consumes the profile and sends semantic commands through a router. Unknown models remain safe/experimental and never silently claim BP1 verification.

**Tech Stack:** Rust/Tauri BLE backend, SolidJS/TypeScript frontend, serde catalog, Rust unit tests, TypeScript/Vite build.

---

### Task 1: Define the APK-compatible profile model

**Files:**
- Modify: `src-tauri/src/protocol/models.rs`
- Modify: `src-tauri/src/ble.rs`
- Modify: `src/lib/ble.ts`
- Test: `src-tauri/src/protocol/models.rs` unit tests

- [ ] Add `NoiseCapability` and `DeviceProfile` data with `mode`, `adaptive`, `environments`, `maxCustomLevel`, `transparencyVoice`, and `verified` fields.
- [ ] Resolve profiles by catalog model ID and firmware family: mode `11` is an app-level mode mapped to wire mode `1`; Lite models cap custom ANC at 3; known full models cap at 5; environments are 101/102/103/108 where supported.
- [ ] Include `deviceProfile` in `BleDevice` and mirror it in TypeScript so the frontend does not reconstruct capabilities from model-name guesses.
- [ ] Add tests for BP1 Pro, Lite, no-ANC, and unknown models.

### Task 2: Add protocol-family routing

**Files:**
- Create: `src-tauri/src/protocol/router.rs`
- Modify: `src-tauri/src/protocol/mod.rs`
- Modify: `src-tauri/src/ble.rs`
- Test: `src-tauri/src/protocol/router.rs`

- [ ] Write failing tests proving BP1 Pro routes to BA34 and unknown/experimental models are marked experimental.
- [ ] Implement `ProtocolRouter` using `ProtocolFamily`, model ID, and v2-wrap requirement.
- [ ] Route semantic noise commands (`Normal`, `TransparencyFull`, `TransparencyVoice`, `CustomLevel`, `AdaptiveEnvironment`) to the selected family; keep the current BP1 BA34 encoder as only the BP1 implementation.
- [ ] Make connection and command paths use the resolved profile rather than calling `Bp1ProAnc` unconditionally.

### Task 3: Match APK connection initialization

**Files:**
- Modify: `src-tauri/src/ble.rs`
- Modify: `src-tauri/src/ble/handshake.rs`
- Modify: `src-tauri/src/protocol/types.rs`
- Test: existing handshake and scan tests

- [ ] Preserve the official control-service filtering and duplicate-entry selection.
- [ ] On first connect, resolve model/serial/firmware, discover the exact write/notify characteristics, perform the APK-style init handshake, then query battery, noise state, EQ, LDAC, and hearing protection through the profile router.
- [ ] Store the profile in connection state and clear it on disconnect.
- [ ] Emit profile/capability state to the frontend before enabling controls.

### Task 4: Refactor frontend state around semantic listening state

**Files:**
- Modify: `src/lib/device.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/HomePanel.tsx`
- Modify: `src/components/ListeningPanel.tsx`
- Modify: `src/styles/components/_home.scss`

- [ ] Replace strength-only state with `ListeningState`: top mode, transparency subtype, adaptive flag, environment, and custom level.
- [ ] Render controls only when supported by the profile; show disabled/experimental state for unknown models.
- [ ] Keep the APK progressive disclosure: three top modes, inline transparency choices, adaptive environment cards, or discrete custom levels.
- [ ] Preserve selections when switching modes and reset safely on disconnect/profile change.

### Task 5: Wire semantic commands end-to-end

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/ble.rs`
- Modify: `src/lib/device.ts`
- Test: `src-tauri/src/protocol/router.rs`, `src-tauri/src/protocol/bp1_pro.rs`

- [ ] Expose one semantic Tauri command accepting the requested listening state, not raw `strength`.
- [ ] Validate requested environment/level against the connected profile before writing.
- [ ] Encode BP1 Pro exactly as APK: `BA34 00 FF`, `BA34 02 FF/voice parameter`, `BA34 01 [1..5]`, and `BA34 01 [101/102/103/108]`.
- [ ] Return clear errors for unsupported commands rather than falling back to BP1.

### Task 6: Verify and document

**Files:**
- Modify: `docs/re/apk-2.14.1/README.md` or create the file if absent
- Modify: `docs/superpowers/plans/2026-07-21-model-profile-flow.md`

- [ ] Run `cargo test --lib` with all tests passing.
- [ ] Run `npm.cmd run build` successfully.
- [ ] Run `git diff --check` and inspect only known line-ending warnings.
- [ ] Document which model profiles are verified, experimental, and scan-only, and explicitly separate APK-derived behavior from hardware-unverified protocol families.
