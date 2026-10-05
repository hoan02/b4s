# Headphone desktop implementation tracker

Source: [approved plan](superpowers/plans/2026-10-06-headphone-desktop-roadmap.md).
Updated: 2026-10-06. This tracker records delivered work separately from hardware acceptance.

## Baseline

- Started from clean `main`, commit `09b4c67`.
- Existing SolidJS/Tauri framework and legacy BLE facade preserved.
- First safety slice: unknown startup query denied, CRC-invalid/truncated wrapped notifications rejected, raw battery salvage removed, invalid ANC/EQ/spatial values rejected.
- Validation: frontend build (includes TypeScript), five-locale parity, 69 Rust library tests and Cargo check passed on the safety slice.
- No hardware verification, firmware manifest, Android captures, or signed installer evidence collected.

## Work packages

| ID | Work | Status |
|---|---|---|
| P0.1 | Chụp baseline Git/build/test; ghi thay đổi đang có, không reset | In progress |
| P0.2 | Ghi firmware BP1 Ultra, Android/Windows version và Bluetooth adapter | External evidence required |
| P0.3 | Ghi phạm vi product và policy đã được người dùng chốt | Complete — architecture/desktop-scope.md |
| P1.1 | Index entrypoints, model guards, SDK/native dependencies | Open |
| P1.2 | Triage JADX errors liên quan; extract resource/native inventory | Open |
| P1.3 | Lần theo family/transport/framing/firmware rules | Open |
| P1.4 | Trace server dictionary/model-param consumers | Open |
| P2.1 | Chuẩn hóa capture plan, local trace format, redaction | Open |
| P2.2 | BP1 capture core features/init/reconnect | External evidence required |
| P2.3 | Replay harness và scripted fake transport | Open |
| P3.1 | Tách BLE discovery/GATT facade khỏi session | Open |
| P3.2 | Tách framing/reassembly khỏi feature decoder | In progress |
| P3.3 | Session lifecycle/generation/cancel/reconnect | In progress — generation guards integrated; actor/cancellation outstanding |
| P3.4 | Queue/correlation/deadline/readback | In progress — bounded serialization/deadline; confirmation outstanding |
| P3.5 | Ưu tiên spike Windows SPP/vendor transport khi U01 xác nhận Ultra cần đường đó | External evidence required |
| P4.1 | Profile v2, validator, migrate BP1 Pro/Ultra explicit | Open |
| P4.2 | Capability resolver/readiness/query planner | Open |
| P4.3 | Device snapshot/error/event contract + compatibility bridge | In progress |
| P4.4 | Scoped persistence, migrations, bounded diagnostic cache | Open |
| P5.1 | App shell/navigation/session store | Open |
| P5.2 | Devices/overview + accurate battery/connect feedback | Open |
| P5.3 | Shared controls pending/error/availability/a11y và Experimental policy | Open |
| P6.1 | ANC/transparency/game, constraints/readback | Open |
| P6.2 | EQ preset/custom/slot with model schema | Open |
| P6.3 | Bass/spatial/codec/hearing constraints | Open |
| P6.4 | Gestures/in-ear, per-side mapping | Open |
| P6.5 | Multipoint/find/device settings | Open |
| P7.1 | Classify headphone-only catalog và legacy migration | Open |
| P7.2 | Adapter của family kế tiếp | Open |
| P7.3 | Hardware validation cho family kế tiếp | External evidence required |
| P8.1 | Windows robustness và accessibility acceptance | External evidence required |
| P8.2 | Signed installer/update + tray/startup/reconnect theo mục 6.4 | External evidence required |
| P8.3 | README/model matrix/diagnostics guide | Open |
| P9 | Cloud/AI/SoundFit/OTA và macOS/Linux | Open |

## Delivery sequence

1. Safety baseline and execution tracker (first PR).
2. Source inventory, capture schema, replay/reassembly boundary.
3. Session generation, command confirmation, typed profiles and persistence.
4. Capability-driven frontend slices and opt-in desktop lifecycle.
5. Catalog/family expansion and Windows release acceptance.

Every PR updates this tracker with implementation, checks, limitations and links. Open means no completion claim. External gates remain open until the named evidence exists; replay never promotes hardware support.

## Required external evidence

BP1 Ultra firmware, Android version/HCI export, Windows version/Bluetooth adapter and U01/U02 captures establish transport and framing. A second-family hardware report and signing/release configuration are separate gates. Continue offline source/replay work while these remain unavailable.

## Increment 2 — framing and capture preparation

- Characteristic-local `NotificationReceiver` separates 789C reassembly from feature decoding. Input is bounded to 4096 bytes; partial frames expire after two seconds and disappear with their stream. CRC/length errors never reach feature state.
- Three focused tests passed: every split point and batching; timeout/reset/overflow; corruption followed by a good frame. Full matrix runs in PR CI.
- Capture guide added; private hardware manifests and original captures ignored. P2.1 remains in progress pending actual identity and repeatable Android export. P2.3 remains in progress pending scripted command/transport timeout and late ACK replay.
- PR: https://github.com/hoan02/b4s/pull/5.

## Increment 3 — session generation

- Extracted `device/session.rs`. Each reset advances an epoch, so reconnecting the same address does not revive old work.
- Notification decode uses the originating device profile even during initialization, before connected identity is published. State updates and compatibility events check the token under the state lock.
- Pollers and delayed demo events check their originating token. Connect attempts are serialized; startup checkpoints and final publication reject cancellation. Disconnect invalidates before asynchronous unsubscribe/disconnect and avoids resetting a newer session afterward.
- Session token test and Cargo check passed. This does not close P3.3: central-event listener ownership, immediate cancellation, command writes across await points and actor lifecycle remain to complete. Windows hardware acceptance is still open.

## Increment 4 — task ownership and cancellation

- Session leases use a watch channel to wake idle notification streams and sleeping pollers on invalidation. Connected command futures also select against cancellation; TX diagnostics reject a superseded session.
- `ble/discovery.rs` owns one adapter event task. It subscribes before scanning, reuses a live task and restarts only after termination. Non-active sibling disconnect events no longer emit active-device disconnect. Scan errors clear their own scanning state; discovery and mock callbacks reject stale scan generations.
- Session cancellation test (including a late subscriber) passed; Cargo check passed after integration. Device commands already handed to the OS cannot be physically recalled; cancellation prevents continuation and success publication, not device-side undo.
- P3.3 remains in progress: explicit session actor, typed event envelopes, timeout policy and full lifecycle/fake-transport coverage are outstanding. P3.4 command ordering/correlation is still open.

## Increment 5 — command admission and state semantics

- `device/executor.rs` bounds connected transactions at 16 and serializes admitted work with a five-second deadline including queue wait. Cancellation drops the transaction; failed/deadline/queue-full outcomes are errors. Three executor tests passed, including proving rejected/expired queued work never invokes transport and admission recovers afterward.
- Generic ANC success no longer resolves from desired state; real ANC state is updated only by a multi-byte report. Empty/invalid game/EQ/LDAC replies no longer fabricate defaults. Ambiguous EQ/LDAC set replies do not publish state; query replies remain decoded. Twenty BP1 decoder tests passed after updating the former ACK-as-state EQ vector.
- Bass/hearing levels outside the existing legacy bound are rejected instead of silently clamped. These bounds remain legacy assumptions, not new hardware evidence; profile-v2 constraint migration is still required.
- This is not command confirmation completion: readback dispatch, response correlation, late-reply quarantine and frontend pending reconciliation remain outstanding. Existing setter return values still describe transport completion until that migration lands.
