# BP1 Ultra BLE transport and controls

On 2026-10-06 a Baseus Bass BP1 Ultra was exercised on Windows 11 Pro
10.0.26200 using btleplug and the B4S debug app. Firmware version is unknown.
The profile is experimental and enables the controls documented below.
EQ/SoundFit, in-ear, multipoint, Find and reset remain unavailable. This observation does
not establish compatibility with every firmware or replace official-app captures.

## Hardware observations

Two distinct BLE addresses advertised the name `BP1 Ultra`. Both exposed the
same vendor service on successful discovery. One advertised that service and
the other omitted it. One discovery attempt returned `NotConnected`; retrying
succeeded. This does not establish whether the addresses represent separate
buds or different identities of the same physical device. B4S keeps both entries
selectable, displays their addresses, and prioritizes the advertised control
service over signal strength.

| Role | UUID / properties |
| --- | --- |
| Service | `53527aa4-29f7-ae11-4e74-997334782568` |
| Write | `ee684b1a-1e9b-ed3e-ee55-f894667e92ac`, WRITE |
| Receive | `654b749c-e37f-ae1f-ebab-40ca133e3690`, READ / NOTIFY |

Reading the receive characteristic returned empty data. Enabling notifications
immediately produced CRC-valid 789C frames. No application handshake or
`#InitState:` write was needed. Create the notification stream before subscribing
to retain these initial reports. B4S declares 789C explicitly for this profile.

## Verified battery exchanges

| Exchange | Bytes |
| --- | --- |
| Earbud query, WithResponse | `789C000A02010102A852` |
| Earbud response | `789C000E02010502640064012C56` |
| Case query, WithResponse | `789C000A020101277393` |
| Case response | `789C000C0201032764006D60` |

The existing CRC/unwrap decoder yields `AA0264006401` and `AA276400`:
left/right/case battery each reported 100%, case not charging. Each query
produced its corresponding notification during a three-second response window.
Additional unsolicited packets decoded to `AA3000` and `AA9002`; their presence
does not authorize feature control. Subscribe, unsubscribe and disconnect all
succeeded. Raw device identifiers and the temporary probe remain in ignored
local captures.

## B4S validation after the fix

The rebuilt desktop app scanned both entries, marked the advertised vendor
service as Control, connected to that entry and displayed real 100% left/case/right
battery values. Startup queries and subsequent battery polls produced the
matching 789C notifications. Disconnect returned to pairing; the saved device
also reconnected after a frontend reload with the existing reconnect preference
enabled. Experimental mode was already enabled in this local app's preferences.
This validates connection and battery on the current device/session only.

## Feature protocol and hardware readbacks

APK 2.17.0.1 source was compared to read/query/write exchanges on the same
Windows earbud. The opcode type table in `HeadPhoneDataResolveManager.b` differs
from the older partial B4S table: BA42, BA74 and BA3F are type 01, BA43 and BA75
are type 03. The CRC fixture tests include actual wire packets, not only packets
produced by B4S itself.

| Control | Bare command / state | Evidence and scope |
| --- | --- | --- |
| ANC | BA34 mode parameter; BA33 -> AA33 mode ANC-type transparency-type custom-level | `NoiseReduceDataModel.n/l`, `NoiseReduceManger.u/s`; Normal, transparency, ANC levels 1/5 and environments 101/102/103/108 written and read back |
| Game | BA24 flag; BA23 -> AA23 flag | `GestureBleManager.e/g`; on/off confirmed after a delayed state report |
| Spatial | BA43 mode; BA42 -> AA42 selected-mode retained-mode | `PanoramicSoundViewModel.x/u`; Music 1, Cinema 2 and Off 0 confirmed; retained selector survives Off |
| Bass Boost | BA54 enabled level; BA53 -> AA53 enabled level | `EarSoundSettingViewModel.N`, `SoundEffectsManger.H/l`; levels 3/5 and Off confirmed; source exposes levels 1..5 |
| LDAC setting | BA75 00 enabled / 01 disabled; BA74 -> AA74 inverted flag | `LdacSettingActivity`; existing enabled setting rewritten and read back; actual Windows audio codec and disabled transition not verified |
| Hearing protection | BA94 enabled threshold; BA93 -> AA93 enabled threshold | `EarSoundSettingViewModel.A/U`; on/off at 80 confirmed; APK thresholds 75/80/85/90/95/100 exposed in UI |
| Gestures | BA22 layout left right (FF preserves side); BA21 layout -> AA21 layout left right | exact BP1 Ultra entries in `assets/gesture/*_v2.json`, `GestureBleManager`; layouts 0..3 queried, one-side unchanged mapping written/read back |

Example CRC-valid feature notifications:

- ANC indoor: `789C000E0201053301660105A8FF` -> `AA3301660105`.
- Normal retains ANC selector: `AA3300660105`; decoder normalizes active parameter to FF.
- Transparency retains ANC selector: `AA3302660105` reports selector 1 (full).
- Ultra transparency selectors are 1 (full) and 2 (voice). Explicit writes use
  `BA340201` and `BA340202`; AA33's third payload byte supplies the active
  transparency selector. The decoder normalizes these to B4S parameters FF
  (full) and 01 (voice), including the command-confirmation comparison.
  The local APK `EarPodNewActivity.initNoiseUI` selects types 1/2 through
  `NoiseReduceManger2.p`; `NoiseReduceDataModel.m` writes the selected type in
  BA34. Voice switching still needs a hardware readback on the user's firmware.
- Spatial Music: `789C000C020103420101E29A`.
- Spatial Cinema: `789C000C02010342020213DA`.
- Bass level 3: `789C000C020103530103264B`.
- Hearing disabled at 80: `789C000C020103930050B70A`.

AA34/AA24/AA43/AA54/AA75/AA94/AA22 success packets are acknowledgements,
not proof of the requested state. Ultra ANC confirmation uses AA33, while Pro
keeps its AA34 state layout. Spatial confirmation compares the selected mode,
not only enabled/disabled. Firmware may retain inactive selectors; the active
mode determines the displayed state. Initialization queries all enabled feature
states, including every reviewed gesture layout.

The gesture profile excludes child actions (such as function 18), which need
additional V2 payload fields. Physical tap behavior has not been exercised for
every allowed function. The Find sound and factory reset were not run.
Queries BA25, BA57 and BA3F produced no response in these sessions, so in-ear,
multipoint and adaptive L/R remain disabled.

## EQ and SoundFit boundaries

The APK proves that Ultra is included in `DeviceManager.Y`: its custom EQ
frequency/Q schema is 65/125/250/500/1000/2000/5000/12000 with Q values
1.2/1.2/1.2/1.4/1.4/1.2/0.8/0.8 (`EqRegulationConstant.DefineSelf`,
`EarEqSelfDefinePresenter.C/E`). BA31 filter payloads are traceable. However,
the inspected Ultra server response has an empty `eq_sound_mode` array; Pro
preset filters and Q=1 custom bands must not be copied into Ultra. This change
keeps EQ writes disabled pending a complete Ultra preset/custom/reset contract
and hardware filter validation. AA3000 alone only proves an index report.
SoundFit previously opened only an unavailable-feature toast, so that placeholder
entry was removed instead of presenting it as a supported control.

Further consumer tracing: `EarEqSelfDefineV2Activity.i1` calls presenter `m`
for Ultra; only Storm 1 calls `n` twice for ANC/non-ANC banks. Thus Ultra custom
EQ has no extra ANC selector: BA31 + slot + filter tuples. Presenter `r` uses
slot 101 for the local flat reset, while cached custom entries also use slot
100. Each tuple is frequency u16 LE, (gain*10+120) u16 LE, (Q*10) u16 LE,
filter u16 LE. An EQ index response cannot read back those coefficients or
recover an existing custom curve. Writing slot 101 before determining how to
restore the original curve would overwrite user data; hardware EQ writes remain
disabled until that recovery path is established.

B4S now carries per-band `qValues` in its model schema, validates their length
and positive finite values, and uses the schema in frontend payloads and backend
authorization. Older profiles with no Q array retain the reviewed Q=1 default.
The Ultra frequency/Q findings above remain source evidence, not authorization
to use the Pro EQ preset/reset contract.

All probe state changes were restored: ANC indoor with custom selector 5, game
Off, spatial Off, bass enabled at 3, LDAC enabled, hearing Off at 80 and unchanged
gesture mappings. The retained spatial selector was restored to Music after the
Cinema test. Private addresses/captures and temporary probe code are excluded
from Git. Hardware acceptance is limited to this device/session; the profile
stays Experimental because firmware scope is unknown.

## Desktop command contract

Live UI testing also found that the tagged `DeviceCommand` enum renamed its
variants but not its struct fields. Frontend `transparencyMode` and `dictSort`
were rejected before BLE dispatch. `rename_all_fields = "camelCase"` fixes the
canonical frontend contract; tests now parse representative listening and custom
EQ requests and reject the obsolete snake_case field. This applies to both Pro
and Ultra command requests.

Live desktop checks also covered Normal -> Transparency -> ANC indoor, Spatial
Music -> Off, and Bass level 4 -> 3. The ANC controller now treats FF as the
default parameter rather than environment 255. Bass options retain stable DOM
identities while device snapshots update. The original settings were restored
and confirmed by device notifications after these UI checks.

Validation: frontend type-check/build, locale consistency, `cargo check`, and
150 Rust library tests passed (`--test-threads=1`).
