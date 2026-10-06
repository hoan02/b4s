# BP1 gesture and in-ear source dossier

Scope: Baseus Android 2.17.0.1 decompiled source (local, gitignored) and the
bundled gesture assets. This dossier records the wire contract for gesture
button mapping and in-ear detection, the model/guard logic, and the exact gaps
that must be resolved before B4S exposes either feature. It is an authored
summary, not a copy of decompiled source.

Source pointers (local dump, not committed):

- `com/control_center/intelligent/utils/GestureBleManager.java` — command builder.
- `com/control_center/intelligent/view/viewmodel/EarHeadSetViewModel.java` — in-ear query/set and state decode (`H`, `U`, `o0`).
- `com/control_center/intelligent/view/fragment/ear/EarphoneFunctionShowFragmentNewUI.java` — in-ear setting reply handling (`s`, around `AA26`).
- `com/control_center/intelligent/view/activity/gesture2/GestureSettingViewModel.java` — v1/v2 selection, query/state parse and setters.
- `com/control_center/intelligent/utils/GestureDataResolveManager.java` — button/layout and function-ID naming.
- `com/base/module_common/manager/DeviceManager.java` (`t0`) — single-button model guard.
- `assets/gesture/gesture_config_function_list.json` and `_v2.json`, `gesture_config_layout.json` and `_v2.json`, `gesture_config_mutex_rules.json` — allowed function sets per model.

## 1. Frame directions

All commands use the same bare/wrapped framing as the rest of the BP1 feature
set: the host writes `BAnn…` and the device answers with the matching `AAnn…`.
Gesture and in-ear therefore ride the existing reviewed framing and GATT
characteristics; no new transport is implied.

| Feature | Host write | Device report | Meaning |
|---|---|---|---|
| In-ear query | `BA25` | `AA25 <state>` | current in-ear detection switch |
| In-ear set | `BA26 <00\|01>` | `AA26 <result>` | disable/enable, then reply |
| Gesture query (v1) | `BA21 <layout>` | `AA21 <layout> <left> <right>` | button/click configuration |
| Gesture set (v1) | `BA22 <layout> <left> <right>` | `AA22 <result>` | write one click action per bud |
| Gesture support (v2) | — | `AA8B <00\|01>` | device advertises gesture v2 when byte = 01 |
| Gesture query (v2) | `BA8C <button> <layout>` | `AA8C …` | v2 layout/state |
| Gesture set (v2) | `BA8D <button> <layout> <function>` | `AA8D <result>` | v2 action write |
| Gesture child query (v2) | `BA8E <button> <layout> <function>` | `AA8E …` | child/parameter list |
| Gesture child set (v2) | `BA8F <button> <layout> <function> <count> [ids]` | `AA8F <result>` | child/parameter write |

`BA23`/`BA24` (mode query/set) live in the same builder but are the already
implemented game mode and are out of scope here.

## 2. In-ear detection

- Query: `BA25` (no payload) — `GestureBleManager.b`.
- Set: `BA26` + `00` (off) or `01` (on) — `GestureBleManager.a`, caller `EarHeadSetViewModel.o0(sn, model, flag)`.
- State: the first payload byte after the opcode is the switch; `EarHeadSetViewModel.H`
  reads `data[2]` and treats `== 1` as enabled. (So `AA25 00` and `AA25 01` are the
  observable off/on states; the app also observes the echo on the set reply.)
- Set reply: `EarphoneFunctionShowFragmentNewUI.s` inspects the first payload byte:
  - `0C` = cannot set while double-connection is active,
  - `0D` = cannot set during a call,
  - otherwise treated as applied (the success path also compares the byte as a state echo).
- Guards: voice-call and dual-connection conflicts; the app shows a specific message
  and does not change the switch. No per-firmware range or single-bud variant was found
  for this command.

Unresolved: whether the set reply byte is an authoritative state or only an echo, the
exact success code set beyond `0C`/`0D`, whether `AA25`/`AA26` are pushed on external
changes, and the model/firmware support list. The reviewed BP1 Pro profile does not yet
declare an in-ear capability.

## 3. Gesture v1

### 3.1 Layout byte ↔ click type

`GestureDataResolveManager` supplies the mapping used by the builder and query:

| Layout byte | Click type | Icon key |
|---|---|---|
| `00` | double click | `ic_c_double` |
| `01` | triple click | `ic_c_triple` |
| `02` | long press | `ic_c_long` |
| `03` | single click | `ic_c_single` |
| `04` | single press | `ic_single_press` |
| `05` | penta click | `ic_c_penta` (not observed in BP1 Pro sets) |

`GestureDataResolveManager.b(earType, action)` maps `(earType, action)` to the internal
case index; `GestureBleManager.f(case, function, …)` turns that into the wire frame.
`earType`: `0` = left, `1` = right, `2` = both/single button.

### 3.2 Query

`BA21 <layout>`. `GestureDataResolveManager`/view models query the layouts that a screen
shows; `EarSimpleGestureViewModel` always queries `BA2100` and `BA2101`, and only adds
`BA2103` for model `Bowie H2`; the full gesture screens query `00`,`01`,`02`,`03` (and
`04` for some models). The suffix is the layout byte only — there is no separate side byte.

### 3.3 State

`AA21 <layout> <left> <right>` (length ≥ 10 hex chars). `GestureSettingViewModel.K`
parses bytes `[4:6]` as layout, `[6:8]` as the first bud's function and `[8:10]` as the
second bud's function. For dual-button models (`DeviceManager.t0 == true`) it produces
two entries: button `0` uses the first function, button `1` uses the second. For the
single-button models listed below (`t0 == false`) it produces one entry with button `2`
using the first function.

`DeviceManager.t0(model)` returns `false` for these single-button/over-ear models (it
returns `true` otherwise and for a null model): Bowie H1, Bowie H2, Bowie P1, Bowie U2,
Baseus Bass EH10 NC, Bowie D05, Bowie H1i, Bowie H1s, C-Mic CM10, Baseus Bass BH1 NC,
Baseus Bass GH03, Bowie 10 Max, Bowie 30 Max, Bowie 35 Max, Baseus Bass BH1,
Baseus P1 Lite, `Bowie  H1S` (double space), Baseus Bass BH1 Air, Bowie H1s Pro,
AeQur 30 Air, Bowie H1 Pro, Baseus Bass BH1 Lite, Baseus P1, Bowie U2 Pro,
Baseus EH10 NC Lite, Baseus Sleep SK1, AeQur N10, Baseus Bowie MH1, Baseus Inspire XH1,
AeQur DS10, AeQur GH02, AeQur VO20, Baseus BH1 NC Lite, Baseus P1x. `Baseus Bass BP1 Pro`
is **not** in this list, so it is treated as dual-button.

### 3.4 Set

`BA22 <layout> <left> <right>` where each side is a function byte or `FF` to leave that
side unchanged. `GestureSettingViewModel.Z` builds exactly this: `layout` then
`left = function, right = FF` for button 0; `left = FF, right = function` for button 1;
`left = right = function` otherwise. `GestureBleManager.f` reproduces the same shapes
for its case indices. `AA22` is the reply; `GestureSettingViewModel.K` wraps it as an
`EarSettingResultBean` and posts the result but does not decode a state value here.

### 3.5 Function IDs

`GestureDataResolveManager.g(int)` maps wire function IDs to actions:

| ID | Action |
|---|---|
| 0 | none |
| 1 | play / pause |
| 2 | previous track |
| 3 | next track |
| 4 | voice assistant |
| 5 | low-latency mode |
| 6 | model-specific noise control (`noise_reduction_tit`; `anc_mode_switch_tit` for H1/H1i/H2/P1/D05) |
| 7 | panoramic sound (spatial) switch |
| 8 | game-effective panoramic sound |
| 9 | rapid mode |
| 10 | light-effect switching |
| 11 | volume up |
| 12 | volume down |
| 13 | bass boost |
| 14 | sleep-helper mode |
| 15 | one-click listen (Ximalaya) |
| 16 | dynamic sound |
| 17 | awaken AI conversation |
| 18 | quick photo |
| 19 | one-click listen (region/login gated) |
| 27 | switch connection mode |
| 28 | microphone on/off |

Button labels: `a(buttonId)` → `0` left, `1` right, `2` MFB, `3` power, `4` volume,
`5` ANC.

### 3.6 BP1 Pro allowed function sets (bundled asset)

`assets/gesture/gesture_config_function_list.json` lists, per click type, the allowed
function IDs for each model group. For `Baseus Bass BP1 Pro`:

| Click type | Allowed IDs |
|---|---|
| single click | 1, 0 |
| double click | 1, 6, 2, 3, 4, 11, 12, 0 |
| long press | 1, 6, 2, 3, 4, 11, 12, 0 |
| triple click | 1, 6, 2, 3, 4, 11, 12, 0 |

So the BP1 Pro UI set is: play/pause, noise control (function 6), previous, next,
voice assistant, volume up/down and none. This asset is the UI allowlist, not proof
that a given firmware accepts every value over the wire.

## 4. Gesture v2

Some models advertise a second gesture protocol through `AA8B` (`01` = supported);
`GestureSettingViewModel.j0` stores that as `mIsGestureV2` and routes later work to the
v2 path.

- Query: `BA8C <buttonId:2hex> <layoutType:2hex>` (`FF` when no layout).
- Set: `BA8D <buttonId> <layoutType> <functionId>`.
- Child query: `BA8E <buttonId> <layoutType> <functionId>`.
- Child set: `BA8F <buttonId> <layoutType> <functionId> <count> [childId…]` (`00` when empty).
- Replies: `AA8C` (state, length ≥ 10), `AA8D` (setting result), `AA8E` (child state, length ≥ 12 with a trailing count), `AA8F` (child result).
- `gesture_config_layout_v2.json` / `gesture_config_function_list_v2.json` and
  `gesture_config_mutex_rules.json` describe v2 layouts, function sets and mutually
  exclusive actions.

The v2 payload details (child-list encoding and mutex semantics) are only partially
traced and are not implemented.

## 5. B4S gap and next steps

No B4S capability, profile field, backend codec or frontend control exists for either
feature. To add them without hardware verification:

1. Decide the protocol version per rated model using `AA8B` support and the bundled
   assets; treat v1 as the baseline and v2 as unresolved until its payload is fully
   traced.
2. Add a model-scoped gesture profile (supported click types, allowed function IDs,
   single/dual button) and an in-ear capability with explicit provenance.
3. Implement `BA21`/`AA21` query-state and `BA22`/`AA22` set, plus `BA25`/`AA25` and
   `BA26`/`AA26`, behind the shared review/capability/firmware authorization.
4. Keep everything `experimental`/`unavailable` until a capture confirms the wire
   values, the `AA22`/`AA26` success semantics, external-change notifications and
   single-bud behavior. The asset allowlist alone is not hardware acceptance.

Remaining unknowns: whether BP1 Pro firmware negotiates v1 or v2; whether function 6
is a toggle or a cycle and its exact payload value; `AA22` success/error bytes; whether
`AA25`/`AA26` are pushed unsolicited; the v2 child encoding; and how the app reconciles
a value changed from another controller.
