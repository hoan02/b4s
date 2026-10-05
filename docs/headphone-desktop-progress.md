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
| P1.1 | Index entrypoints, model guards, SDK/native dependencies | In progress — transport dispatch indexed; feature/native inventory outstanding |
| P1.2 | Triage JADX errors liên quan; extract resource/native inventory | In progress — reproducible local inventory tool; method triage/resource extraction outstanding |
| P1.3 | Lần theo family/transport/framing/firmware rules | In progress — exact transport/framing guards indexed; firmware/callback tracing outstanding |
| P1.4 | Trace server dictionary/model-param consumers | In progress |
| P2.1 | Chuẩn hóa capture plan, local trace format, redaction | Open |
| P2.2 | BP1 capture core features/init/reconnect | External evidence required |
| P2.3 | Replay harness và scripted fake transport | Open |
| P3.1 | Tách BLE discovery/GATT facade khỏi session | Open |
| P3.2 | Tách framing/reassembly khỏi feature decoder | In progress |
| P3.3 | Session lifecycle/generation/cancel/reconnect | In progress — generation guards integrated; actor/cancellation outstanding |
| P3.4 | Queue/correlation/deadline/readback | In progress — bounded serialization/deadline; confirmation outstanding |
| P3.5 | Ưu tiên spike Windows SPP/vendor transport khi U01 xác nhận Ultra cần đường đó | External evidence required |
| P4.1 | Profile v2, validator, migrate BP1 Pro/Ultra explicit | In progress — connection descriptors migrated; firmware/feature evidence outstanding |
| P4.2 | Capability resolver/readiness/query planner | In progress — backend feature authorization; per-feature evidence/readiness outstanding |
| P4.3 | Device snapshot/error/event contract + compatibility bridge | In progress |
| P4.4 | Scoped persistence, migrations, bounded diagnostic cache | Open |
| P5.1 | App shell/navigation/session store | In progress — ordered session store and listener cleanup; feature controller extraction outstanding |
| P5.2 | Devices/overview + accurate battery/connect feedback | Open |
| P5.3 | Shared controls pending/error/availability/a11y và Experimental policy | Open |
| P6.1 | ANC/transparency/game, constraints/readback | Open |
| P6.2 | EQ preset/custom/slot with model schema | In progress |
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

## Increment 6 — readback and typed snapshots

- EQ preset/index, game, LDAC and hearing setters now keep the transaction open for a matching query-state observation. Unrelated/ACK-only/wrong-session replies do not finish it. Unsupported EQ indexes without a state decoder reject before write. Deadline and disconnect return errors; no blind retry is added.
- Battery refresh waits for an AA02 observation inside the serialized transaction, including unchanged values. Case-only/unrelated notifications cannot complete it. Exact full battery layouts accept 0–4%; embedded-pattern salvage and percentage-based rejection are removed. Case 0% also remains a value.
- Backend `DeviceSnapshot` v1 carries session, receipt revision, model, optional confirmed state and timestamped nullable battery readings. Repeated values advance revision/freshness. Tauri getter/event and `src/bridge/deviceSnapshot.ts` preserve the legacy bridge while preparing frontend migration.
- Fifteen device tests passed, covering matching query versus ACK, session isolation, unknown/zero distinction and unchanged receipt. Decoder and frontend checks recorded with this increment's commit.
- AA/BA has no request ID: matching readback proves the observed state in a session, not which write caused it. Late-reply handling, profile/firmware applicability, ANC/other feature confirmation and frontend adoption remain open; no hardware promotion.

## Increment 7 — frontend snapshot adoption

- `src/stores/deviceSession.ts` owns device/session/revision ordering and async-operation identity. Stale sessions, duplicate revisions, another device and disconnected events cannot overwrite current state. Same-address reconnect and backend session rollover invalidate pending completions.
- App adopts typed snapshots for real battery/ANC/EQ/game/LDAC. Legacy listeners are retained for demo or features awaiting snapshot fields. Async listener registrations resolving after unmount are immediately disposed. Mock snapshots are explicitly marked and seeded from demo data.
- Battery view models/components use nullable percentages: unknown displays a dash, true 0% displays 0%. Switching device/session resets listening drafts and pending errors. EQ/game retain confirmed state during application with visible pending/inline errors; repeated submits are disabled. Shared OperationStatus has status/alert roles and five-locale text.
- Frontend build and locale parity passed; session reducer tests cover ordering, device switch, reconnect and backend rollover. Cargo check passed after deviceId/mock snapshot contract updates. CI includes frontend session tests. CI for commit 03daee6 completed successfully on Windows/Ubuntu; subsequent commits still in progress at inspection.
- UI screenshots, scaling/screen-reader acceptance, all-feature unknown/pending semantics, full feature-controller extraction and hardware flows remain open. No P5/P6 completion claim yet.

## Increment 8 — explicit connection profiles

- Profile schema v2 separates control transport, framing, UUIDs, handshake, init policy, firmware inventory and provenance. Validator rejects unknown schema, missing descriptors/UUIDs, invalid UUIDs and active handshake on an unresolved transport. BP1 Pro metadata now shares its reviewed E37F notify UUID with runtime; no E37A guess is used.
- BP1 Ultra is migrated to an explicit scan-only profile: source suggests 789C but U01/firmware/Windows transport is not verified. Its legacy row is also passive, preventing reactivation if a profile disappears. Legacy models without a reviewed transport cannot control real hardware; demo remains available.
- Connect sends only the selected profile handshake, never probes bare/wrapped fallbacks or learns framing from write success/unchecked notification. Subscription and writes require the profile's exact characteristic/service. Battery BA02/BA27 use the same framing. Ambiguous WithResponse failures are not blindly retried as WithoutResponse.
- Eight catalog tests passed and Cargo check passed. Firmware-version lists remain empty pending capture; BP1 Pro's existing compatibility status is not a new firmware-wide hardware claim. Per-feature evidence/constraint migration, readiness confirmation and transport acceptance remain open.

## Increment 9 — backend feature authorization

- Resolved DeviceProfile capability flags come only from reviewed profiles; public metadata and legacy-name heuristics cannot supply runtime permissions. Every listening/feature encoder runs shared transport/capability/firmware authorization before serialization. Missing transport, disabled capability, unreviewed experimental model and mismatched explicit firmware scope reject intent.
- Find support is declared separately. Custom EQ validates band layout, finite gain/Q and profile gain limits before serialization. Advanced sound UI hides controls absent from the backend profile, including LDAC/hearing on BP1 Pro.
- Two capability tests and seven router tests passed during integration; frontend typecheck passed. Added a custom EQ invalid-layout/nonfinite boundary test for CI. Firmware ranges, per-feature evidence descriptors, conflict/readiness checks and an eligible Experimental mode preference remain open. Experimental is disabled by default; there is no force flag.


## Increment 10 — source-correct preset EQ

- Android consumer tracing corrected an earlier assumption: AA42 is spatial state and AA43 is an ACK. EQ selection query/readback is BA30/AA30. Increment 6's generic EQ decoder restriction is superseded by raw wire-index observations.
- BP1 Pro preset writes now use BA31 with the seven catalog presets and source-derived parametric filters. Backend resolves UI preset IDs and rejects indexes absent from the model. Serialization uses source truncation and LE16 fields; no fabricated ANC byte or fixed eight-filter limit is added to presets.
- Frontend lists the selected model's presets, reconciles raw dictSort and hides missing preview curves instead of displaying invented curves. Source dossier: `docs/protocol/bp1-pro-eq-source.md`.
- P1.4 and P6.2 remain in progress. Custom EQ, firmware applicability, persistence and hardware verification are outstanding; source evidence does not close acceptance gates.

- Validation for increment 10: all 88 Rust library tests passed; frontend typecheck/Vite build and `git diff --check` passed. Source-filter byte vectors cover truncation, little-endian encoding and Q above an 8-bit field; ACK/spatial replies cannot become EQ observations. UUID presentation assertion now compares canonical case.


## Increment 11 — source-correct BP1 Pro custom EQ

- Traced EarEqSelfDefinePresenter A/B/C/F/G/m/n/r/w and the activity's model branch: BP1 Pro sends BA31 + index 101 + eight LE16 filter records, without the Storm 1 ANC selector. Default frequencies use the model's existing 100–10000 Hz layout, Q=1 and peak type=1. Unsupported slot/ANC/Q/filter intents reject before encoding.
- Custom serialization shares the preset filter encoder, removing truncation of the filter count and the old rounded/clamped 8-bit field assumptions. Custom apply now requires current-session AA30 index 101 before returning success; frontend retains confirmed state during pending/failure and rejects stale completion.
- Index 101 identifies current custom selection; it does not prove filter content or hardware slots. Android custom lists are locally cached; desktop saved profiles are local drafts, not a verified device slot inventory. Full model editor/persistence/reset reconciliation and hardware acceptance remain open.

- Validation: eight router tests passed, Cargo check and frontend typecheck/Vite build passed, whitespace check clean. Added a dedicated invalid custom slot/selector/filter test for CI without another local full-suite run.


## Increment 12 — confirmed equalizer selection and reset

- Extracted model-specific readback selection into `features/equalizer/selection.ts`. AA30 101 restores custom selection after reconnect; recognized indexes select the current catalog preset; missing/unrecognized indexes remain unknown instead of fabricating Classic. Local custom drafts do not become device filter readback.
- Reset now retains confirmed selection/draft until successful readback, shares pending/error behavior and rejects stale session completion. Custom apply/reset buttons prevent repeated submissions while pending.
- Five frontend session/selection tests passed, covering unknown index, capability scope and model preset mapping. Full controller/editor extraction and hardware filter/persistence evidence remain open.


## Increment 13 — equalizer operation controller

- Extracted command admission, pending/error publication and session/operation epoch guards into `features/equalizer/controller.ts`. Preset/custom/reset use the same controller; device/session reset invalidates old work without allowing its finalizer to clear a newer operation.
- Both command completion and snapshot refresh are checked before UI success. Real-device active selection comes from snapshots; demo retains explicit local publication. Preset success labels come from the model catalog.
- Seven frontend tests and typecheck passed. Race tests cover duplicate write admission, session rollover during refresh and old completion after reset/new operation. Whitespace check passed. No Rust changes or repeated local full build; PR CI handles the build matrix. P5.1 remains in progress for other feature slices.


## Increment 14 — catalog-driven custom editor

- Equalizer receives frequencies, gain bounds and custom permission from the selected model. Sliders and draft serialization use the model frequencies; a missing/mismatched schema rejects before invoking the backend. Custom controls are unavailable without capability.
- Saved drafts are scoped to device, model and frequency layout, so an equal-length but different schema cannot reinterpret previous gains. Existing legacy-key drafts remain stored but are not silently migrated. Loading rejects nonfinite/out-of-range values and wrong band counts; reactive schema changes reload their own drafts.
- Frontend typecheck/Vite build and whitespace check passed. Local draft capacity remains an application preference, not a hardware-slot claim. Q/type editing for other models, explicit draft migration, visual accessibility acceptance and hardware persistence remain open.


## Increment 15 — remove speculative frontend EQ defaults

- Removed unused twelve-preset fallback tables, fabricated display curves, index-to-Classic fallback and global fixed band definitions from the frontend EQ utility. Catalog is now the sole preset/frequency source.
- Custom drafts initialize with the selected model's band count and reset on device/frequency-layout changes. Draft loading requires explicit model count/gain limits; unknown models begin with no bands.
- Latest branch CI was pending at inspection (run 37357610361); no wait or completion claim. Focused frontend typecheck validates this cleanup; hardware acceptance and remaining roadmap packages remain open.


## Increment 16 — accurate equalizer copy and storage errors

- Removed outdated user-facing BA43/reset claims in all five locales. Preset copy describes model selection; custom copy distinguishes local drafts from confirmed device application. Band count text uses the selected schema.
- Save/delete publish the updated draft list only after localStorage succeeds. Failed storage operations preserve the visible list and expose a localized alert; failures no longer escape the event handler or falsely appear saved.
- Locale parity (237 keys), frontend typecheck and whitespace checks passed. Durable settings migration/export, storage corruption diagnostics and UI/hardware acceptance remain open.


## Increment 17 — transport dispatch source inventory

- Added Android transport source dossier with independent exact-name H0 (Classic dispatch) and Q0 (789C framing) inventories, connect/write entrypoints and local dump pointers.
- Identified priority BLE write divergence: normal/alternate paths send the framing result, while the priority branch passes original sourceData. Constructor-provided RFCOMM UUID and caller/capture coverage require further tracing; no fixed SPP UUID or Windows support is inferred.
- P1.1/P1.3 now explicitly in progress; native/error/hash/firmware/callback inventory remains outstanding. Documentation only; no redundant local build. Latest inspected CI run 37357844957 was pending.


## Increment 18 — Classic socket source lifecycle

- Traced the constructor's SPP UUID, per-address connect admission, installed socket RX/TX owner, cancellation and SDK receive callbacks. RX emits arbitrary chunks from a 2048-byte read buffer; write callback indicates transport completion only.
- Android queue release compares a two-byte RX prefix against request metadata. This cannot serve as B4S command/state confirmation. Added explicit stream reassembly/EOF/cancel/reconnect acceptance requirements to the transport dossier.
- UUID source is now resolved in source, while actual BP1 Ultra firmware/Windows endpoint remains unverified. Application callback decoder/readiness and queue timing semantics remain open. Documentation-only evidence increment; no local build or CI wait.


## Increment 19 — reproducible local Android inventory

- Added Python CLI to hash APK/XAPK/DEX/native inputs, inspect native entries inside APKs without extraction and locate decompiler error/undecompiled-method markers without source snippets. Output is restricted to ignored repository `.tmp`.
- Synthetic checks passed for native-entry SHA256 and marker locations/source exclusion. Real dump inventory exited successfully: 3 hashed inputs, 106 APK native entries and 1410 decompiler markers. Marker counts may include two markers per failed method and are not the original 508-error run count. Input package/version/tool provenance, original JADX run errors, resource inventory and affected-method DEX verification remain open.
- Documentation explains local-only output and distinguishes marker counts from JADX run errors. No frontend/Rust rebuild for this tooling change.


## Increment 20 — headphone method/native triage

- Reviewed local marker inventory against EQ, transport/framing and Classic SDK paths. DeviceManager.l(String) is undecompiled and directly feeds min/max slider thresholds in NoiseReducePopWindowV2; this is an explicit source gap for ANC range reconstruction.
- Indexed native-loader ownership for Thingclips BleLib, generic CommandFactory native-lib and BES OTA JNI. Native presence alone is not a dependency of the BP1 offline path; call graphs remain required.
- Added source triage dossier with exact local pointers and recovery requirements. No speculative range/command changes, local builds or broad decompiler reruns. P1.2 remains in progress pending DEX recovery and resource/native caller coverage.
