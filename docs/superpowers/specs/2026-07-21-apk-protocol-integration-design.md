# Baseus APK protocol integration design

## Goal

Use the reverse-engineered Baseus 2.14.1 APK to make B4S a complete local
companion for supported Baseus audio devices: first-time discovery/binding,
model identification, product presentation, GATT setup, state synchronization,
and every listening control currently exposed by B4S. Preserve model-specific
framing and clearly separate verified behavior from best-effort behavior.

## Scope

- Preserve the current Tauri/SolidJS/Rust architecture.
- Implement the APK's first-connect flow locally: BLE scan, advertisement
  parsing, model/service filtering, serial derivation, GATT service discovery,
  notify subscription, init-state query, bind-state handling, timeout, and
  reconnect.
- Add a model catalog with canonical model id, display name, BLE name patterns,
  capability flags, service/characteristic strategy, color variants, and an
  image field with a bundled fallback. Do not pretend the APK contains product
  images: its `icon`/`iconLarge` values come from server metadata.
- Keep product metadata loading separate from BLE control. The app must remain
  usable offline with the local catalog and fallback image; a future/optional
  metadata provider may fill exact remote icons and colors.
- Keep one protocol command abstraction and add builders for the commands
  found in the APK: ANC, EQ, spatial, game mode, bass boost, LDAC, hearing
  protection, battery, and find buds.
- Route every command through the existing BLE write path so bare BA/AA and
  789C+CRC wrapping remain centralized.
- Decode acknowledgements and state notifications for the controls B4S can
  represent: battery, ANC, EQ, game, bass, LDAC, hearing protection, and
  spatial.
- Replace UI-only LDAC, hearing protection, and bass behavior with real BLE
  calls and retain explicit error feedback when a model does not support a
  command.
- Add protocol unit tests for command bytes, v2 wrapping, and notification
  decoding.
- Add deterministic scan-parser tests for manufacturer data, service UUID
  filtering, serial canonicalization, and model matching.
- Add connection-state tests covering: no matching service, GATT discovery
  failure, notify setup failure, `#InitState:` timeout, `Init State`, and
  `Already Configured`.
- Keep APK/decompiler output out of the application bundle and repository
  history; commit only small, human-reviewed protocol findings when useful.

## Evidence-based command map

| Feature | APK command | B4S action |
|---|---|---|
| ANC | `BA34` + mode + level | retain and verify payload |
| EQ | `BA43` + preset/custom payload | retain presets; implement custom payload path |
| Spatial | `BA43` + mode | retain, with model capability guard |
| Game mode | `BA24` + `00/01` | retain |
| Bass boost | `BA54` + enabled + level | replace current EQ-based fallback |
| LDAC | `BA74`, `BA75` + `00/01` | add backend command and notification state |
| Hearing protection | `BA94` + enabled + level | add backend command and notification state |
| Battery | `BA02`, `BA27` | retain and merge left/right/case |
| Find buds | `BA100201` | retain |

## First-time connection flow found in the APK

1. Check Bluetooth, location, and platform permissions, then scan for up to 30
   seconds. Deduplicate by address/derived serial and ignore devices whose
   advertised name/model is not recognized.
2. Parse AD structures, service UUIDs, and manufacturer data. For models that
   require it, accept only the advertised Baseus service UUID
   `53527AA4-29F7-AE11-4E74-997334782568`.
3. Derive the canonical serial from the two six-byte manufacturer values. The
   APK sorts/joins them and reverses byte order for model families that require
   it; the parser must preserve both raw values and the canonical form.
4. Connect with LE transport, wait for connection state `2`, delay 200 ms, and
   call service discovery. Use server-provided per-model UUIDs when available;
   for self-UUID devices use the first service and its first two
   characteristics (notify first, write second).
5. Enable notifications by writing the CCCD, then consider the link ready only
   after the descriptor write succeeds. Queue writes so only one request is
   active at a time; clear the matching queue item only after a notification
   with the expected opcode arrives.
6. After a 1-second settling delay, send the UTF-8 bytes of `#InitState:`. A
   distribution event for the same serial is interpreted as `Init State` or
   `Already Configured`; the first case binds with `relieveBind=1`, the second
   with `relieveBind=0`. If no state arrives within 30 seconds, disconnect and
   report a recoverable failure.
7. Persist the serial/model/address mapping and the selected product metadata,
   then use the same transport for queries and controls on reconnect. Special
   families may expose a separate classic-Bluetooth or vendor transport and
   must not be forced through the earbud GATT path.

## Product image and model metadata strategy

The APK's add-device response contains `model`, `prodName`, `icon`,
`iconLarge`, `colorList`, `videoPic`, and `videoUrl`; the exact image URLs are
server-provided and are not embedded as per-model assets in the APK. B4S will
therefore use this precedence:

1. local model-specific asset supplied in the repository;
2. cached metadata/image from an explicitly configured provider;
3. the bundled `default_ear_pic.png` fallback.

The catalog will expose provenance (`local`, `cached`, `remote`, `fallback`)
so the UI can avoid showing a generic image as if it were an exact product
render. Exact Baseus images will only be added when supplied by the user or
returned by an allowed metadata source; the reverse-engineered APK alone is
not evidence for inventing those URLs.

The APK also contains many other commands, but they are outside B4S's current
listening UI or require separate device families. They will remain catalog
evidence rather than being guessed into the product.

## Data flow

Scan result -> advertisement parser -> model catalog -> GATT connector ->
notify subscription -> init-state handshake -> persisted device session.

Frontend action -> Tauri command -> Rust protocol command builder -> model-aware
framing (`BA/AA` or `789C + CRC16`) -> serialized BLE write queue.

BLE notify -> unwrap frame -> decode opcode/payload -> update shared BLE state
-> emit typed Tauri event -> update UI.

## Error handling and compatibility

- A successful GATT write is not treated as proof that the earbud accepted the
  command; link health and decoded acknowledgements remain separate.
- Unknown or malformed notifications are logged and ignored without changing
  the UI state.
- Product metadata/image failures must not block BLE connection or control.
- Binding must be opt-in and explicit in the UI; a device may be controlled in
  local mode without a cloud account when the protocol allows it.
- Mock mode emits deterministic state changes for UI testing but never claims
  to be a live device.
- Model entries remain `verified`, `experimental`, or `scanOnly`; adding an
  opcode does not automatically promote a model.

## Verification

- Rust unit tests for all new command builders and decoders.
- Existing frontend typecheck/build.
- Static comparison of generated command hex against the APK decompile.
- If a physical device is available, manual checks for Live link, command
  acknowledgement, and battery/notification updates on BP1 Pro/Ultra.
