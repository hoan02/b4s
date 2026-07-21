# Baseus APK protocol integration design

## Goal

Use the reverse-engineered Baseus 2.14.1 APK to make every listening control
currently exposed by B4S send the correct protocol command, while preserving
model-specific framing and clearly separating verified behavior from
best-effort behavior.

## Scope

- Preserve the current Tauri/SolidJS/Rust architecture.
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

The APK also contains many other commands, but they are outside B4S's current
listening UI or require separate device families. They will remain catalog
evidence rather than being guessed into the product.

## Data flow

Frontend action -> Tauri command -> Rust protocol command builder -> model-aware
framing (`BA/AA` or `789C + CRC16`) -> BLE write characteristic.

BLE notify -> unwrap frame -> decode opcode/payload -> update shared BLE state
-> emit typed Tauri event -> update UI.

## Error handling and compatibility

- A successful GATT write is not treated as proof that the earbud accepted the
  command; link health and decoded acknowledgements remain separate.
- Unknown or malformed notifications are logged and ignored without changing
  the UI state.
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

