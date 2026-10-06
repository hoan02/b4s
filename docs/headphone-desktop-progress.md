# Headphone desktop implementation tracker

Source: [approved plan](superpowers/plans/2026-10-06-headphone-desktop-roadmap.md).
Updated: 2026-10-06. This tracker records delivered work separately from hardware acceptance. Updated user direction: replace legacy runtime paths with one clean architecture; allow explicit one-time data migration only.

## Baseline

- Started from clean `main`, commit `09b4c67`.
- Existing SolidJS/Tauri framework retained. The legacy BLE facade is being replaced; no compatibility runtime path is allowed.
- First safety slice: unknown startup query denied, CRC-invalid/truncated wrapped notifications rejected, raw battery salvage removed, invalid ANC/EQ/spatial values rejected.
- Validation: frontend build (includes TypeScript), five-locale parity, 69 Rust library tests and Cargo check passed on the safety slice.
- No hardware verification, firmware manifest, Android captures, or signed installer evidence collected.

## Work packages

| ID | Work | Status |
|---|---|---|
| P0.1 | Chụp baseline Git/build/test; ghi thay đổi đang có, không reset | Complete — clean base and original checks recorded |
| P0.2 | Ghi firmware BP1 Ultra, Android/Windows version và Bluetooth adapter | External evidence required |
| P0.3 | Ghi phạm vi product và policy đã được người dùng chốt | Complete — architecture/desktop-scope.md; updated direction supersedes legacy-facade policy |
| P1.1 | Index entrypoints, model guards, SDK/native dependencies | In progress — transport dispatch indexed; feature/native inventory outstanding |
| P1.2 | Triage JADX errors liên quan; extract resource/native inventory | In progress — reproducible local inventory tool; method triage/resource extraction outstanding |
| P1.3 | Lần theo family/transport/framing/firmware rules | In progress — exact transport/framing guards indexed; firmware/callback tracing outstanding |
| P1.4 | Trace server dictionary/model-param consumers | In progress |
| P2.1 | Chuẩn hóa capture plan, local trace format, redaction | In progress — repeatable U01–U09 guide and scrubbed trace schema exist; actual capture review/validation remains open |
| P2.2 | BP1 capture core features/init/reconnect | External evidence required |
| P2.3 | Replay harness và scripted fake transport | In progress — synthetic pipeline plus confirmed-transport fake; full GATT/capture replay outstanding |
| P3.1 | Thay BLE discovery/GATT facade bằng transport/session mới | In progress — BLE mutable state and singleton now have a dedicated runtime owner; scan/discovery, connection, command, and GATT I/O remain separate modules; actor lifecycle, injected transport interface, and event ownership remain open |
| P3.2 | Tách framing/reassembly khỏi feature decoder | In progress — notification reassembly now requires the profile's declared framing |
| P3.3 | Session lifecycle/generation/cancel/reconnect | In progress — generation guards integrated; actor/cancellation outstanding |
| P3.4 | Queue/correlation/deadline/readback | In progress — bounded serialization/deadline; confirmation outstanding |
| P3.5 | Ưu tiên spike Windows SPP/vendor transport khi U01 xác nhận Ultra cần đường đó | External evidence required |
| P4.1 | Profile v2, validator, migrate BP1 Pro/Ultra explicit | In progress — schema v2 is mandatory; only reviewed profiles authorize control; Ultra remains passive pending transport evidence |
| P4.2 | Capability resolver/readiness/query planner | In progress — backend feature authorization; per-feature evidence/readiness outstanding |
| P4.3 | Device snapshot/error/event contract thay thế API cũ | In progress — snapshot schema and scan/connection/link/connecting DTO versions are validated at frontend boundaries; closed link enum; session/sequence envelopes and consistent typed error semantics remain open |
| P4.4 | Scoped persistence, migrations, bounded diagnostic cache | In progress — auto-reconnect settings are versioned; custom EQ storage is model/device scoped with legacy-array migration and validation; bounded diagnostic cache remains open |
| P5.1 | App shell/navigation/session store | In progress — ordered session store, runtime subscriptions and find-buds workflow now have dedicated owners; remaining feature-controller extraction and navigation/accessibility acceptance are open |
| P5.2 | Devices/overview + accurate battery/connect feedback | In progress — battery unknown/zero and link-level status are explicit; duplicate ANC environment controls and hidden dead UI paths removed; device inventory/visual acceptance outstanding |
| P5.3 | Shared controls pending/error/availability/a11y và Experimental policy | Open |
| P6.1 | ANC/transparency/game, constraints/readback | Open |
| P6.2 | EQ preset/custom/slot with model schema | In progress |
| P6.3 | Bass/spatial/codec/hearing constraints | In progress — binary bass readback and advanced snapshots; per-model hearing policy exists but feature remains disabled; spatial policy outstanding |
| P6.4 | Gestures/in-ear, per-side mapping | Open |
| P6.5 | Multipoint/find/device settings | Open |
| P7.1 | Classify headphone-only catalog và legacy migration | In progress — static 124-model resolver removed; one-time exact-name ID migration now covers 112 catalog identities; 12 historical names have no exact catalog identity and remain inert |
| P7.2 | Adapter của family kế tiếp | Open |
| P7.3 | Hardware validation cho family kế tiếp | External evidence required |
| P8.1 | Windows robustness và accessibility acceptance | External evidence required |
| P8.2 | Signed installer/update + tray/startup/reconnect theo mục 6.4 | In progress — tray lifecycle, bounded Quit cleanup, and opt-in startup/reconnect preferences; installer/signing and Windows acceptance remain external |
| P8.3 | README/model matrix/diagnostics guide | In progress — architecture, catalog, and protocol guides now describe the single modern profile-driven runtime; full model/feature evidence matrix remains open |
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


## Increment 21 — recover ANC range instructions

- Targeted JADX 1.5.6 raw-instruction fallback recovered DeviceManager.l without rerunning the whole APK. Exact model-name branches return minimum 1 and maximum 3/5/10; BP1 Pro/Ultra target 5.
- Existing BP1 Pro maxCustomLevel=5 agrees with source, so no speculative runtime expansion was made. The default Android range 10 does not grant permission to unknown B4S models.
- Raw output remains ignored; source triage dossier records tool/method/branch interpretation and distinguishes same-engine raw inspection from an independent decompiler result. Capture/firmware acceptance, range wire/readback and remaining failed-method/resource inventory stay open.


## Increment 22 — advanced audio snapshot adoption

- Snapshot contract adds nullable observed bass boost and timestamped hearing enabled/level. Session reset clears both; zero/disabled observations remain real values and unchanged hearing receipts advance freshness/revision.
- Frontend real-device bass/hearing now uses ordered session snapshots; compatibility listeners only publish these values in demo mode. Spatial lacks a reviewed state decoder and is not fabricated into this snapshot.
- Two focused snapshot tests and frontend typecheck passed, whitespace clean. Full unknown/pending UI semantics, bass/spatial/hearing range/correlation policy and hardware acceptance remain open under P4.3/P6.3.


## Increment 23 — shared confirmed operation and advanced toggles

- Moved the tested operation controller into `features/shared/confirmedOperation.ts`. EQ and advanced sound now share admission/session/refresh guards without duplicating pending/error logic.
- LDAC/hearing no longer optimistically toggle or invert state on failure. Real-device values come from snapshots; pending disables repeated toggles and inline status/error is visible. Demo publication remains explicit. Session reset invalidates old advanced operations.
- Frontend typecheck and seven session/controller tests passed. Bass/spatial still need their own readback/constraint migration, and full unknown-state UI plus hardware acceptance remain open.


## Increment 24 — unknown advanced toggle state

- Frontend LDAC/hearing values are nullable, reset to unknown on session loss and follow nullable snapshots. Missing readback no longer initializes a confirmed off state.
- Supported controls show localized unknown-state text and accessible mixed checkbox state until observations arrive. Users can submit an explicit toggle, while pending retains the observed/unknown value and rejects repetition.
- Frontend typecheck, 238-key five-locale parity and whitespace checks passed. Screen-reader/scaling acceptance and remaining advanced/listening unknown states remain open.


## Increment 25 — reject bass error payloads

- Source receive handling distinguishes AA53 query from AA54 set replies and AA54 error codes such as 0C/0D. Existing decoder clamped arbitrary payloads to level 3; removed that behavior and rejected empty, out-of-range, inconsistent and extra-byte layouts.
- Three focused bass tests passed, including all error/invalid layouts. Legacy compact/enabled-level decoding remains compatibility behavior, not completed source verification; one-byte success ambiguity, actual AA53 decode and query confirmation remain open.
- Source pointers: EarphoneFunctionShowFragmentNewUI receive routing around 3022/3028 and error handling around 950–978; EarHeadSetViewModel BA53 query around 463. No new bass hardware claim or UI success-policy completion.


## Increment 26 — source-correct bass toggle/readback

- Source ResultHandle.i/j and Setting.i establish BA54 00/01 write, BA53 query, AA53 boolean state and AA54 success/error ACK. Replaced the invented enabled+level write and four-level UI; backend rejects levels above 1 and AA54 never publishes bass state.
- Bass setter waits for matching AA53 observation in the serialized current-session transaction. UI uses nullable snapshot state and shared confirmed-operation pending/error behavior; unsupported multi-level intents reject instead of clamping.
- All 63 protocol tests and frontend typecheck passed; five-locale On label added. Firmware/conflict/readiness and hardware verification remain open. Increment 25's retained legacy layouts are superseded by this source-traced query-only decoder.


## Increment 27 — authorized startup bass query

- Added explicit QueryBassBoost BA53 startup request so reviewed BP1 Pro sessions can observe initial bass state. Startup EQ/bass/LDAC/hearing queries use the shared backend feature authorizer, not public/marketing capability flags.
- Extracted common control authorization for transport/review/firmware checks; startup also requires model/profile identity equality. Unknown/unreviewed/mismatched profiles cannot trigger speculative battery/feature queries.
- Three initialization tests passed. Added marketing-flag, firmware/identity mismatch and BA53 byte coverage for CI without another local suite. Startup readiness/correlation and captured initialization order remain open.


## Increment 28 — hearing threshold source contract

- Traced hearing UI threshold list and setter: 75/80/85/90/95/100 dB, with -1 encoded as FF. AA94 is ACK; AA93 is state query and has region/model UI guards.
- Recorded mismatch with legacy B4S setter 0–3/default 1 as an explicit implementation gap. Existing BP1 Pro hearing capability remains disabled; no global permissive range change or unsupported model activation.
- Added source dossier with consumer pointers and per-model threshold/sentinel/readback/capture requirements. Documentation only, no redundant build. P6.3 remains in progress.


## Increment 29 — model hearing threshold constraints

- Added optional hearing profile with explicit threshold list, FF-preservation permission and provenance. Validator requires constraints for an enabled hearing capability and rejects missing/duplicate/non-source threshold values.
- Backend setter replaces the 0–3 legacy bound with exact model threshold/sentinel membership. Frontend toggling preserves the observed threshold instead of inventing level 1; absent threshold returns an unknown-state error.
- Five catalog tests and frontend typecheck passed. Added missing/valid/invalid threshold-schema test for CI. No model capability is newly enabled; threshold editor, sentinel semantics and model/firmware acceptance remain open.


## Increment 30 — spatial enabled readback boundary

- Added BA42 query and exact AA42 boolean decoder, separate from EQ index and AA43 ACK. SpatialEnabled observations enter nullable session snapshots; startup queries require reviewed spatial capability. Frontend adopts observed enable state without inferring a mode.
- Cargo check and frontend typecheck passed. Existing EQ spatial-reply test now asserts the separate spatial event; startup expected plan includes its authorized query. No local full-suite rerun; CI covers tests.
- Source PanoramicSoundViewModel.E/H uses BA42 query and BA43 mode write. Its firmware-dependent mode list and exact selected-mode readback still need tracing. Setter confirmation, full nullable spatial UI and EQ/bass conflict handling remain open; boolean readback cannot prove Music/Cinema/Game selection.


## Increment 31 — spatial enable confirmation

- Spatial writes query BA42 and wait for exact AA42 enabled state in the current serialized session; AA43 ACK cannot complete them. This confirms enablement only, not selected mode.
- UI uses shared operation admission/pending/error guards and retains observed enable state on failure. EQ conflict disable waits for readback and checks session before continuing; stale completion cannot apply EQ to a newly selected session.
- Cargo check/frontend typecheck passed. Added query-versus-ACK/wrong-state/old-session confirmation test for CI. Mode selection remains a local requested choice until a model-specific mode readback is verified; nullable spatial UI, firmware mode constraints and hardware acceptance remain open.


## Increment 32 — unknown spatial state and conflict gate

- Spatial enabled state is nullable in the UI, resets to unknown and adopts nullable snapshots. Unknown is displayed with localized text and mixed checkbox state instead of confirmed off.
- EQ conflict handling treats an unknown spatial state as unresolved when spatial capability exists; disable/readback completes before proceeding. Models without spatial capability are exempt from this conflict route.
- Fixed mutable epoch declaration in the new confirmation test. Three confirmation tests and frontend typecheck passed; whitespace clean. Latest inspected CI run 37394930493 was pending. Selected-mode evidence, backend conflict policy and accessibility/hardware acceptance remain open.


## Increment 33 — overview capability visibility

- Overview spatial/game/EQ/find controls now require resolved backend profile capabilities. Advanced audio navigation appears only when bass/LDAC/hearing is available; unknown profiles no longer expose actions that the backend necessarily rejects.
- Existing backend authorization remains authoritative. This is capability visibility, not readiness/conflict/firmware evidence completion or Experimental enablement.
- Frontend typecheck and whitespace checks passed; no repeated local build/tests for this reversible view gating. Visual/device acceptance and remaining unavailable/planned feature presentation remain open.


## Increment 34 — game unknown state and operation reuse

- Game mode UI uses nullable observed state, with localized unknown text/mixed checkbox before readback. Session reset no longer fabricates off.
- Game command admission/pending/error and completion move to the shared confirmed-operation controller. Session is checked after both command and refresh; an old refresh cannot publish success in a new session. Removed duplicated handler lifecycle code.
- Frontend typecheck, seven session/controller tests and whitespace checks passed. ANC draft/confirmed state, query readiness and hardware acceptance remain open under P5/P6.


## Increment 35 — integrated synthetic notification replay

- Added a pipeline replay through connection-local receiver, CRC/framing, BP1 decoder and session-scoped expected state. Every split point of a synthetic wrapped bass observation is exercised after a corrupt frame and bare set ACK.
- Correct readback confirms only the matching desired state; ACK cannot decode as bass, corrupt input cannot reach confirmation and reconnect invalidates the old observation. Focused replay test passed and runs with the existing CI library suite.
- This is synthetic pipeline evidence only. Fake transport write/queue/deadline scripting, late-response quarantine and capture-based replay remain open under P2.3/P3.4; no hardware promotion.


## Increment 36 — transport-independent readback awaiter

- Extracted shared asynchronous state awaiter into device/confirmation. BLE feature/battery transactions subscribe before TX and delegate matching to it; executor retains deadline/cancellation ownership.
- Scripted broadcast tests cover wrong state, ACK opcode, old session, eventual matching readback, channel close and lag overflow. Readback loss returns an error rather than accepting later residual messages. Five confirmation tests passed, whitespace clean.
- Full fake write transport/deadline harness, late-response causality policy and captured replay remain open. The helper confirms observations within session only; it does not add request IDs to AA/BA.


## Increment 37 — honest overview link status

- Home status now renders explicit offline state and uses localized unknown text for unrecognized LinkHealth levels instead of labeling them connected. Status is exposed through a polite live region for assistive technology.
- Frontend typecheck and whitespace checks passed; no additional local build/test. Full device inventory/connect progress, battery charging presentation, visual/accessibility acceptance remain open under P5.2/P8.1.


## Increment 38 — speaker exclusion from new discovery

- Public catalog discovery now filters products whose regional category paths all classify them as speaker series. A regionally mixed classification stays eligible to avoid a false exclusion.
- The five reviewed speaker-only products (AeQur 30 Air, DS10, N10, VO20, Sleep SK1) already have explicit legacy registry entries. Filtering only applies while merging new public-catalog discovery, so legacy IDs and preferences remain resolvable.
- No local tests run per current task guidance; CI will validate the Rust change. P7.1 still needs broader family/type audit and user-preference migration evidence before closure.


## Increment 39 — hide speakers from headphone pairing

- Catalog taxonomy audit covered 129 audio models and all seven variant leaf categories; only the same five products are consistently speaker-only.
- The BLE scan payload now marks catalog-classified speakers, and the pairing UI omits them from new headphone choices. Stored-device auto-reconnect still reads the raw scan inventory and can resolve legacy IDs; unknown or regionally mixed products remain visible.
- Pairing empty/searching feedback now follows visible candidates, so seeing only filtered speakers does not leave a blank scan panel.
- Contributor catalog guidance now distinguishes the offline audio metadata catalog from headphone pairing and documents the speaker and legacy-reconnect policy.
- Added CI regression coverage for the five category exclusions, mixed regional categories and all legacy IDs. Local tests remain unrun by request; visual pairing review and the remaining catalog/family gates remain open.


## Increment 40 — scripted confirmed-transport boundary

- Extracted the write→query→matching-state sequence behind `ConfirmedTransport` and connected the production GATT writer through that seam. Subscription still happens before writes and the existing command executor retains deadline/cancellation ownership.
- CI coverage scripts wrong-opcode ACK, matching readback, write failure, disconnect, deadline and a late ACK followed by a fresh transaction. This covers state confirmation behavior without hardware; it cannot establish command causality for a late matching state because AA/BA has no request ID.
- Local tests remain unrun by request. Full connection/discovery GATT facade and captured notification replay remain open under P2.3/P3.1.


## Increment 41 — correct CI decoder and scan-only regressions

- The first CI run passed frontend checks and Cargo check; Rust tests exposed an outdated ACK-only fixture that still rejected valid AA42 spatial state, plus Sleep SK1 inheriting the legacy GATT UUID default.
- Updated spatial decoder coverage to accept AA42 state while rejecting ACK AA43, and routed all catalog-derived scan-only entries through a helper that clears transport UUIDs. This preserves Sleep SK1 recognition without guessing a control connection.
- CI rerun required. No local tests run; validation remains pending for the correction.


## Increment 42 — desktop lifecycle and opt-in preferences

- Added a system tray with Open, live device status and Quit. Closing the main window hides it only when tray setup succeeds; explicit Quit makes bounded best-effort scan/find cleanup and disconnect before exit.
- Added OS-backed start-at-login preference and persisted auto-reconnect opt-in, both off by default. Reconnect remains one scan for the last supported device, and failure to save an OS preference is reported.
- The reconnect decision is captured at app launch: enabling it during a running session takes effect next launch, while disabling it cancels any remaining reconnect opportunity in the current session.
- Rust `cargo check` passed on the Windows workspace after fixing the tray close-event signature and scan-only catalog mutability. CI found that resolving the new plugin had advanced the Rust Tauri crate to 2.12 while the JS API is 2.11; the Rust crate and its locked runtime/build family are now aligned to 2.11.5. Full CI is required for Linux/schema/frontend integration; tests were not run locally.
- Signed installer/update artifacts, tray behavior on supported Windows versions, startup migration behavior, and the P8.1 sleep/resume/accessibility acceptance remain open. This increment does not close P8.2 or claim release readiness.


## Increment 43 — correct desktop support and troubleshooting docs

- Updated the five READMEs to describe reconnect as opt-in/off by default, the 12-second one-shot search, separate start-at-login preference, and close-to-tray/Quit behavior. Removed the unsupported claim that BP1 Ultra is verified for control; its transport remains scan-only pending evidence.
- Added a desktop troubleshooting guide with current scan/connect steps, startup/tray behavior, issue-reporting fields, and explicit diagnostic collection/privacy limits. Updated the model catalog guidance to distinguish reviewed profile data from hardware verification.
- Documentation links and claims were reviewed against current code and the support tracker; no local tests run. Full per-model/per-feature evidence matrix and P8.1/P8.2 external acceptance remain open.


## Increment 44 — scoped EQ preference migration

- Custom EQ remains keyed by device address, model ID and band layout. Stored data now uses a versioned envelope, validates IDs/labels/gains/band count, caps the saved list to the UI's two-preset limit, and migrates the legacy array shape on read without discarding usable values if storage is unavailable.
- Locale preference reads and writes now tolerate browser storage denial; language switching remains available for the current session. Theme and reconnect preferences already handle storage errors independently.
- No local tests run. The bounded diagnostic cache and broader persistence migration matrix remain open under P4.4.


## Increment 45 — honor reconnect launch boundary

- Auto-reconnect eligibility is captured when the app initializes. Enabling the preference mid-session applies on the next app launch; disabling it in Settings clears the current launch's eligibility. Returning to pairing can no longer unexpectedly start a reconnect attempt after a same-session opt-in change.
- No local tests run; CI covers the frontend build. Full opt-in reconnection and cancellation acceptance on Windows hardware remain open under P8.1/P8.2.


## Increment 46 — replace compatibility-first direction

- User direction now requires one clean runtime architecture with no legacy runtime fallback or dual execution. The plan and desktop-scope ADR supersede earlier instructions to keep `ble.rs`/compatibility listeners as a bridge. P3.1 and P4.3 now require replacing those paths; P7.1 requires an explicit one-time ID migration followed by removal of legacy resolution.
- Data-preserving migrations remain allowed when versioned and one-shot. Unknown model/firmware/transport remains unavailable; it cannot route through a generic adapter.
- This records a policy change, not implementation completion. Replacement slices and regression evidence remain required. The pending EQ type guard passed `npx tsc --noEmit`; no tests were run locally.


## Increment 47 — explicit catalog and single-path BLE selection

- Removed the static legacy model registry, inferred protocol/capability defaults, substring identity matching, and generic Baseus AA/BA protocol family. Reviewed profile JSON is the only control source; public catalog entries remain passive scan metadata, and schema v2 is mandatory.
- BLE connect now targets the selected entry and exact profile service/write/notify UUIDs. Same-name Windows entries remain separately selectable; generic characteristic probing, alternate notify selection, sibling auto-connect, and name-based peripheral re-resolution were removed. Disconnect unsubscribes only the reviewed notification characteristic.
- Notification reassembly and decoding now require the profile's declared bare-AA/BA or 789C framing. Unresolved framing is passive, and wrapped framing cannot accept bare notifications.
- Removed the unused legacy `set_anc_mode` command and frontend heuristic that fabricated ANC range/adaptive support from model IDs. The typed listening-state command and resolved profile constraints are the only control path.
- `cargo test --lib` passed: 101 tests. One stale provenance assertion failed in the first run and was updated; the complete rerun passed. `git diff --check` passed. BP1 Ultra transport, firmware scope, full BLE runtime extraction, stored-ID migration, frontend contract replacement, and hardware acceptance remain open.


## Increment 48 — one device-state event contract

- Replaced per-feature device events with the versioned `device://snapshot` stream for battery, ANC, EQ, game, spatial, bass, LDAC, and hearing state. Demo-session observations publish the same snapshot contract; the frontend no longer listens to individual device events. Removed the direct battery getter and unused connected/disconnected/device/raw events.
- Removed default mode/range arguments from custom EQ, hearing protection, and listening commands; callers must send explicit values, and invalid transparency enums now return errors.
- Rust library suite passed 101/101 on the same change set before a dead empty branch was removed; latest `cargo check`, `npx tsc --noEmit`, and whitespace checks pass. CI is pending for this slice. Link-health, scan, and connection streams remain separate operational state and still need a versioned typed contract.


## Increment 49 — split BLE runtime by responsibility

- Extracted profile-driven scan/discovery coordination, connection/subscription/readback lifecycle, device commands, and exact-profile GATT writes into `ble/scanning.rs`, `ble/connection.rs`, `ble/commands.rs`, and `ble/transport.rs`. Tauri handlers now call the domain modules directly. `ble.rs` retains shared runtime state and cross-cutting snapshot/link helpers and is down from about 2,060 to 702 lines.
- This is a module ownership refactor over the single strict execution path; it introduces no legacy wrapper or alternate transport. `cargo check` passes; the full Rust library suite is running. Central state ownership/actor extraction, injected transport abstraction for full connection tests, and typed scan/link/connection lifecycle events remain open.


## Increment 50 — explicit one-time model ID migration

- Added a frozen table mapping 112 historical model IDs to canonical runtime/catalog IDs by exact full product identity. Startup migrates the remembered-device record and model-scoped custom-EQ keys once; bytes are copied only when the destination is absent, and no mapping participates in scan or live identity resolution.
- Twelve historical names have no exact identity in the current public snapshot; their stored data is left untouched and inactive rather than assigned by substring or similarity. The explicit table is based on exact normalized full-name equality against the current snapshot/reviewed profiles; `npx tsc --noEmit` passed.


## Increment 51 — versioned link and lifecycle DTOs

- Scan status, connection state, and link health now expose `contractVersion: 1` from both command responses and their matching events. Link health uses a closed `LinkLevel` enum with the five supported values; arbitrary strings are rejected at the Rust boundary and no longer admitted by the frontend type.
- Frontend command and event adapters validate version 1 before delivering these states. Added serialization coverage for the versioned link contract and enum values. `cargo check`, TypeScript typecheck, and `git diff --check` passed; repository-wide `cargo fmt --check` still reports existing formatting differences outside this slice, so only the edited Rust file was formatted.
- No local test suite was run; CI will exercise the serialization test. This advances P4.3 but does not close it: event sequence/session envelopes, typed error semantics, and remaining command DTO migration are open.


## Increment 52 — central BLE state ownership

- Moved the mutable BLE state aggregate, initialization/reset logic, and synchronized singleton into `ble/runtime.rs`. The root `ble.rs` now consumes the owner while discovery, scanning, connection, command, and transport modules share the same state without an alternate runtime.
- `ble.rs` is now about 600 lines. `cargo check` and whitespace validation passed; no local test suite was run. P3.1 remains open for session actor/cancellation ownership, injected transport, and event stream lifecycle.


## Increment 53 — frontend runtime subscription ownership

- Moved scan-independent connection/link/snapshot listener registration and disposal into `features/devices/runtimeSubscriptions.ts`. Registration is sequential, aborts when the owner is disposed, and immediately unregisters listeners that resolve after unmount.
- `App.tsx` now supplies state handlers and owns only the returned disposer; ordering/session behavior is unchanged. `npx tsc --noEmit` passed before the final disposal-race guard adjustment; no local test suite was run. CI will validate the final change.
- P5.1 remains open for extracting feature controllers and verifying navigation/accessibility behavior.


## Increment 54 — align contributor docs with the current architecture

- Rewrote the architecture guide around reviewed profiles, passive public metadata, exact identity, explicit family codecs, strict transport selection, snapshot state and versioned runtime DTOs. Updated model-catalog guidance to remove retired legacy-registry/generic-family claims and describe the current `bp1`/`unknown` family boundary.
- Updated stale Rust module and router comments that still described the removed experimental compatibility path. No build was needed for this documentation/comment-only slice; full per-model evidence documentation remains open under P8.3.


## Increment 55 — remove ACK-as-hearing-state compatibility path

- The BP1 decoder now accepts hearing state only on source-traced AA93. AA94 remains an ACK/error opcode and can no longer update the confirmed snapshot even if it carries two payload bytes. Added regression assertions for short and state-shaped AA94 responses.
- Replaced stale protocol references that claimed BP1 Ultra was a verified target, called the BP1 Pro profile “legacy compatibility,” retained a model registry, or retried an alternate handshake. The strict profile/evidence boundary is now explicit.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib` is left to PR CI per current guidance; no hardware behavior is claimed.


## Increment 56 — remove unreachable listening UI paths

- Removed the hidden ANC strength slider/state and two permanently hidden find-bud dialogs. Removed duplicate adaptive-environment cards so each available environment appears once.
- Updated the catalog identity comment to describe exact product normalization rather than a retired app registry. `npx tsc --noEmit` and `git diff --check` passed; visual/hardware acceptance remains open under P5.2/P8.1.


## Increment 57 — extract find-buds controller

- Moved find-buds active/confirmation state, start/stop commands and localized feedback into `features/find-buds/controller.ts`. `App.tsx` now composes the controller with the home view and dialog; reconnect/disconnect reset feature-local state.
- `npx tsc --noEmit` and `git diff --check` passed. No test suite was run locally; CI validates the frontend build. Other listening/equalizer controllers and hardware stop/cancel behavior remain open.


## Increment 58 — validate all active BLE event versions

- The in-progress connection event now carries a versioned `ConnectingState`; its frontend adapter validates the version before exposing the selected device ID. Device snapshot invoke/event adapters now reject unsupported `schemaVersion` values too.
- Removed the unused `ble://bind-state` event that guessed binding actions by searching arbitrary notification bytes for English text; no frontend consumer existed.
- `npx tsc --noEmit`, `cargo check`, and `git diff --check` passed. P4.3 remains open for session/sequence envelopes and structured command/error contracts; no compatibility event path remains for these states.
