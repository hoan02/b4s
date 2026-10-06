# Model parameter and dictionary consumers

Inspected 2026-10-06 against the locally extracted Baseus 2.17.0.1 APK. This
note records authored conclusions and source pointers only. APKs, raw parameter
responses, and decompiled source stay in ignored local directories.

## `getModelParams`

`ControlApi` declares several variants of
`GET /app/category/getModelParams`, keyed by `model` and optionally `color`,
`version`, `alarm`, or `channel`. The parsed response is the broad
`EqRegulationBean`, not an EQ-only DTO. The consumers found in headphone flows
include:

| Response field | Observed consumer | Meaning and limitation |
|---|---|---|
| `eq_sound_mode` | `EarEqDefaultRegulationActivityNewUi` and its presenter | Model-scoped display presets and custom-EQ entries. Entries carry labels, `dictSort`, display/image URLs, and a packed `description`; the values are adapted by model-aware presenter logic before display/use. Empty arrays are handled by the UI and do not establish that the earbud lacks EQ. |
| `gestureSmallImgs2` | `GestureDataViewModel.queryGestureMainIcon` | Model-specific gesture imagery. This is presentation metadata, not the gesture command/action table. |
| `operate_guidepage` / `operate_guidepage_us` | `EarphoneSettingFragment`, `QuickGuideActivityV2`, and settings activities | Model-specific phone-app instructions. Presence alone does not establish a desktop action or protocol capability. |
| `earphoneClearList` | `EarCleanViewModel.getEarphoneClearFunction` | Cleaning feature configuration in the phone app. The local response for both BP1 probes is empty; this is not evidence of a desktop command. |
| `splash` | Add-device/search flows | Optional product/onboarding image metadata; callers test its URL together with operation-guide availability. |
| `audio_file` | Shared sleep-helper UI | Audio catalogue data consumed by phone UI; not a control packet or evidence of a Windows playback path. |

The locally retained read-only parameter responses show `eq_sound_mode` with
seven entries for BP1 Pro and an empty array for BP1 Ultra. Both contain model
asset metadata such as gesture images and guides; both have empty cleaning
lists. These snapshots are observations for one app version and probe context,
not a complete contract for every firmware, locale, or account.

## `dictByName`

`ControlApi` exposes `GET /app/homepage/dictByName` and
`dictByNames` variants. Inspected response types include account profile,
protection, and pay dictionaries. One traced `dictByName` consumer loads
`default_account_info` and looks up the `avatar` label for the home profile.
That path is account presentation data. The current trace did not establish a
model-scoped headphone control dictionary name or prove that all dictionary
consumers are covered; `dictByName` must not be treated as a source of runtime
capabilities without tracing the request and command consumer.

## Packaged gesture configuration

The APK contains two gesture-layout/function schemas plus a mutex-rules file.
`GestureDataViewModel` loads the `_v2` layout and function assets; the older
head-gesture and gesture view models load the original layout/function assets.
The records are model-scoped and associate button/layout types with allowed
function IDs. Model/locale/account and supported-function guards further filter
what the phone UI presents. The newer view model stores selections under a key
derived from serial number and model in MMKV. These assets describe UI choices
and persistence; they do not by themselves provide the command opcode mapping,
ACK/readback behavior, or an authorization list for B4S.

The archive also has `assets/command/query.json`, with per-row `function`,
`command`, `mustDevice`, and `supportDevice` fields. No source consumer for this
file was found by filename in the targeted JADX output. Its contents and exact
runtime owner remain unresolved; do not use it as a protocol or capability
source without locating that consumer.

## B4S implications

- Keep EQ wire IDs, display curves, image URLs, and device capabilities as
  separate profile fields. Server labels and URLs do not define a wire value.
- Keep gesture command maps and action IDs unsupported until their model guards,
  packet mapping, and state readback are traced.
- Treat operation guides, cleaning entries, splash images, and sleep audio as
  phone-app configuration until there is independent evidence of a local
  desktop behavior.
- Do not fetch these endpoints during discovery or connect. Any future metadata
  integration must be an explicit user action with a documented cache and
  privacy boundary.

## Evidence limits and follow-up

Source pointers in the local extraction are `ControlApi`,
`EarEqDefaultRegulationActivityNewUi`, `GestureDataViewModel`,
`EarphoneSettingFragment`, `HeadGestureFunctionChooseViewModel`,
`QuickGuideActivityV2`, `EarCleanViewModel`, `SleepHelperViewModel`, and
`HomeFragment`. JADX output can be incomplete; the consumer list is a targeted
trace, not an exhaustive APK inventory. Dynamic dictionary request names,
remaining `getModelParams` fields, model/firmware guards, and any failed methods
on these exact call paths still need tracing. No feature permission or model
profile was changed from this source review.
