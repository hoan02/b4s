# APK 2.17.0.1 feature and model parity audit

Reviewed 2026-10-07 against the supplied XAPK, local JADX source/resources,
the 124-model discovery snapshot and current B4S frontend/backend. This is a
source-level implementation audit; no new hardware exchanges were performed.
It is not a certification of every APK branch or firmware. Decompiled source,
APK assets and raw responses remain outside Git.

## Conclusion

A fuller standard model catalog can be built from this APK, but B4S does not
currently have feature parity with it. All 124 names have evidence; only BP1 Pro
and BP1 Ultra have runtime control profiles and adapters. Recognition coverage,
feature evidence, implemented commands and hardware acceptance are four distinct
properties. Finding a model name or an opcode must not turn on a capability.

## Implementation matrix

| Area | APK evidence to trace | Current B4S coverage / limitation |
| --- | --- | --- |
| Model identity | `DeviceManager.DEVICE_MAP`, exact model branches, public catalog | 124 recognized candidates, seven experimental/verified control profiles (BP1 Pro/Ultra plus EP10 Ultra, M4s, MS1, M3s, EP10 Pro, the last five not hardware verified); extracted alias mappings are evidence, not automatically installed runtime aliases |
| Transport and framing | `BluetoothDataWriteManager.Companion.a/b/c`, `DeviceManager.H0/Q0`, `HeadPhoneDataResolveManager` | The APK controls all seven profiled models over Classic Bluetooth SPP (`H0`); B4S uses BLE GATT with bare AA/BA and 789C framing on its own hardware evidence. No Classic Bluetooth control transport in B4S |
| Battery / device state | Headphone data decoder, model-specific state consumers | Pro/Ultra decoders and startup queries exist; no universal layout for all models |
| ANC / transparency | `NoiseReduceDataModel`, `NoiseReduceManger2`, `EarPodNewActivity` | Two reviewed layouts; Ultra AA33 differs from Pro; voice write/read normalization implemented, Ultra voice hardware transition still pending |
| EQ preset / custom | `EarEqSelfDefinePresenter`, `EqRegulationConstant`, model params | Pro reviewed preset/filter schema; schema 3 declares the current slot/bank explicitly; Ultra EQ remains disabled; not a universal EQ bank/reset contract |
| Game / spatial / bass | `GestureBleManager`, `PanoramicSoundViewModel`, `EarSoundSettingViewModel` | Implemented for reviewed models; schema 3 supplies per-model bass limits within the implemented adapter range |
| LDAC / hearing threshold | `LdacSettingActivity`, `EarSoundSettingViewModel.A/U` | Ultra settings implemented; setting readback does not establish the OS's actual audio codec; no model-wide capability inference |
| Gesture legacy | `GestureBleManager`, legacy gesture assets | BA21/BA22 layouts and reviewed function allowlists exist; Pro feature is experimental-only |
| Gesture V2 / child actions | `gesture2/GestureSettingViewModel.H/U/V/W/...`, V2 assets | Missing support negotiation BA5D and V2 feature paths BA8C/8D/8E/8F; runtime gesture schema has no button-specific child-action contract |
| In-ear / multipoint / defaults / adaptive L/R | Corresponding APK managers/settings consumers | Commands, queries and gates exist for Pro as source/replay-based experimental features; Ultra paths remain disabled where readbacks were absent |
| Call enhancement | `EarSoundSettingViewModel.P`, BA4C | No feature intent, state decoder, confirmation or UI contract |
| Battery enhancement | `EarSoundSettingViewModel.R`, BA6D | No feature intent, state decoder, confirmation or UI contract |
| Sound balance | `EarSoundSettingViewModel.M/W`, BA7A/7B | No parameter schema, query/write state path or UI contract |
| Connected-device management | `SmartConnectionViewModel` BA3B/3C/3D/3E and BA96/97 | Pro boolean multipoint is not equivalent to device-list, device-action and intelligent-connection management |
| Find | `FindEarPhoneActivity`, settings consumers | Pro has a reviewed feature path; Ultra disabled; no proof of general side/volume/duration/position parity |
| Firmware update | Single-ear upgrade and vendor OTA SDK trees | Tauri updater updates B4S, not earbud firmware; no earbud OTA implementation |
| Personalized sound / hearing / ANC | `MimiLogicViewModel`, `HearingHealthActivity`, `EarPersonalizedNoiseActivity`, AI tuning classes | No corresponding complete feature contract; hearing protection toggle is not personalized hearing calibration |
| AI translation / recording / conversation | AI activity/viewmodel trees | No corresponding implementation; these may depend on Android audio, accounts and remote services and are a separate desktop product scope |

The 789C opcode type table already lists some unsupported commands. That table
only categorizes framing: it is not an implementation of their features.
Likewise, a command enum member does not prove that a model authorizes it or
that its state can be confirmed.

## Important model-specific findings

`BluetoothDataWriteManager` uses `H0(model)` to select the official app's Classic
Bluetooth SDK and `Q0(model)` to select 789C wrapping. BP1 Pro and Ultra both
appear in H0's positive branches. B4S's Ultra GATT path was independently
verified on the current device in earlier sessions. These findings can coexist:
the official app's preferred transport is not proof that an alternative verified
GATT path is invalid, nor permission to give every model the Ultra UUIDs.

The gesture assets have function branches for 120 models, but a V2 asset is not
proof that every firmware uses V2 wire commands. The APK queries support and has
separate legacy/V2 paths. B4S currently implements the reviewed legacy mapping
subset, not that full negotiation. Child function selections need their own
payload and state contract; flattening them to function IDs loses behavior.

The Pro profile's overall `verified` label must be read alongside its
`experimentalFeatures`: gesture, in-ear, multipoint, restore defaults and
adaptive L/R remain separately gated. Empty firmware allowlists currently mean
there is no enforced firmware restriction; they do not establish firmware-wide
acceptance. Ultra remains experimental and has unknown firmware scope.

## Standard catalog design

Keep an evidence catalog separate from the runtime control catalog. Every model
should record canonical name, explicit aliases/product identity, region and APK
version provenance. Every feature should distinguish `unknown`, `unsupported`,
`sourceReviewed`, `implemented` and `hardwareVerified`, with source/hash/line or
asset JSON pointer plus firmware scope. A false runtime capability alone cannot
explain whether the feature is absent or simply unreviewed.

A runtime profile should select transport, framing, handshake, initialization
and a tested protocol adapter independently. Add typed, per-model schemas for
sound limits, gesture protocol/button/layout/child actions, EQ slots and bank
selectors, connection modes and feature dependencies. Adapter code owns byte
encoding/decoding; JSON supplies reviewed data and constraints. No arbitrary
Java execution, inferred hash-code branches or unreviewed command strings belong
in the runtime profile.

A profile must pass this acceptance sequence per feature:

1. Trace the model/firmware condition from the UI consumer to the query and write.
2. Trace transport, framing and notification decoding, including ACK vs state.
3. Define value ranges, defaults, dependent settings and recovery behavior.
4. Add replay fixtures for query/write/state, retained selectors and errors.
5. Verify hardware readback and the intended physical behavior; record scope.

Do not copy Pro filters, Ultra UUIDs or shared gesture lists across models without
these steps. Firmware update and server-backed AI need separate implementations,
not additional capability booleans.

## Suggested implementation order

First standardize the evidence/status schema and per-model sound/gesture/EQ
constraints. Then finish the two existing models, prioritizing gesture V2
negotiation/child actions, Ultra EQ recovery and the missing sound controls where
the exact model branch permits them. Next add another model only after tracing
its entire transport and state contract; reuse an adapter only when that contract
matches. Handle device management, calibration, OTA and AI as separate projects
with their own acceptance criteria.

See [model evidence review](model-evidence-review-2.17.0.1.md) for extraction
coverage and [BP1 Ultra hardware evidence](protocol/bp1-ultra-ble.md) for the
limitations of earlier readbacks. No runtime capabilities were expanded by this
audit.

The subsequent [structural refactor](model-standardization.md) implemented schema
3, explicit sound/EQ/gesture contracts, evidence statuses and separated command
constraints. Remaining functional gaps in this matrix are still unimplemented.

## Validation of current implementation

`bun run build` (including TypeScript checking) and `cargo check` passed.
Rust library tests after the catalog/image work: 166 passed, one hardware smoke test ignored.
Frontend session, preferences, error formatting, reconnect, logging, storage and
model-contract, ID migration and image cache tests are included in the release checks.
These checks validate the existing implementation and contracts; they do not
establish parity with the missing APK features or acceptance on other earbuds.

## Cross-check of suspected mismatches (2026-10-07)

A second pass over the APK compared four B4S assumptions with their consumers:

- **AA37 (restore defaults)**: superseded by the REA re-audit below. The
  `00` check belongs to `EarphoneSettingFragment`, which these models do not open;
  their settings screens treat `01` as success.
- **BA5E**: `PanoramicSoundViewModel.G` writes `BA5E01`/`BA5E00` to declare
  support; there is no `AA5E` consumer. The wrong "capability query" comment in
  `protocol/types.rs` was corrected. B4S does not send BA5E.
- **LDAC polarity**: superseded by the REA re-audit below; the call sites agree
  that `01` enables LDAC.
- **`#InitState:`**: only used by the add-device flow (`SearchDevicesAddViewModel`),
  not by connected earbuds. The BP1 Ultra GATT path needs no such write.

## REA re-audit (2026-10-13)

The APK was re-analyzed with [REA](https://github.com/morluto/rea) 6.4.0 and its
pinned headless JADX engine (jadx-headless-mcp 0.7.1, SHA-256
`6e5eacf5...`), which decompiles single methods on demand and records an evidence
ID per result. Methods JADX could not restore (`NoiseReduceDataModel.f`,
`DeviceManager.l`) were read as smali or JADX fallback output. Five areas were
compared against B4S: transport/framing, battery/ANC, EQ/sound, gestures and
device settings, and per-model capability gating. Each conclusion below cites the
consumer that decides it.

Fixed in B4S:

- **LDAC polarity was inverted on write and readback.** `LdacSettingActivity`'s
  `DisTouchAbleSwitchButton` does not toggle on touch, so `m1` sends `BA75 00` to
  turn LDAC *off*; `AA74 01` checks the switch. The fragment and
  `HeadPhoneMainActivity.F` agree. B4S now sends `01` to enable and reads `01` as
  enabled.
- **AA37 success is `01`.** `EarPhoneSettingActivity`/`V2Activity` (the screens
  all seven profiles use) report every other code as an error. B4S waited for
  `00`, so a successful reset timed out. It now completes on `01`, fails fast on
  other codes and re-runs the startup queries.
- **AA34 is a write status, not noise state.** `NoiseReduceManger.s` reads byte 0
  as `01` success / `00` failure and re-queries `BA33`; state comes from `AA33`
  (`NoiseReduceManger.u`). The BP1 Pro family now confirms through `BA33`,
  queries it at startup and fails fast on a rejected status.
- **789C receive no longer invents frames.** B4S searched every CRC-valid packet
  for a `0xAA` byte and also had a fallback layout; neither exists in
  `HeadPhoneDataResolveManager.Companion.d/e`, and a payload or length byte equal
  to `0xAA` produced fake frames. Sub-frames are now kept in device order instead
  of being deduplicated by opcode.
- **Per-model limits:** `DeviceManager.d` caps the bass slider at 3 for BP1
  Ultra, EP10 Ultra and M4s (MS1 keeps 5). `NoiseReduceDataModel.f` gives the Pro
  family 104/102/103/101 (no 108), the Ultra family 104/102/103/101/108 and MS1
  only 104, each with Custom 1-5 (`DeviceManager.l`). The Pro family has one
  transparency mode. Custom EQ gain is clamped to +/-6 dB in
  `DragChartLineViewV2`, separately from preset filters.
- **Over-claims removed:** BP1 Pro adaptive L/R (`BA3F` is queried only for
  Inspire XC1) and EP10 Pro presets (copied from BP1 Pro; its server parameters
  are not captured). Short aliases `m3s`, `m4s` and `ms1` were removed because
  they are not `DEVICE_MAP` keys and "MS1" is also a Tuya device ID.
- **Spatial readback:** `AA42` reports the selected mode for the BP1 Pro family
  too, so Cinema can be confirmed.

Added from the same evidence: per-side find (`BA10 00|01|02 xx`) on all seven
profiles; wind noise for the Ultra family and LDAC for the Pro family, shown only
after the earbuds answer `AA51`/`AA74` as the app does; AA8B gesture contract
detection that blocks legacy `BA22` writes when firmware selects V2; one-sided
gesture writes with `FF` for the untouched side; and localized
`EarSetFailedUtil` reasons for rejected writes.

Still open: Classic SPP transport, gesture V2 encoding and child actions
(function 18), Ultra-family EQ, call/battery enhancement, sound balance,
device management, OTA and AI services. The BP1 Pro `BA0500` handshake has no
APK earbud source but is kept pending a capture. Every change above is
source-level; none is a hardware acceptance claim.
