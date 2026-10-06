# Repeatable headphone captures

Use the approved U01–U09 sequence. Start with U01 startup and U02 reconnect
before choosing BP1 Ultra transport. A name or successful write is insufficient.

1. Record model, firmware, Baseus app version, Android OS, Windows OS and
   Bluetooth adapter in a private local manifest. Unknown fields stay unknown.
2. Record pairing state, bud/case placement, settings and competing connections.
3. Export Android HCI logs using the actual vendor/OS-supported path. Keep
   originals in `.local-captures/`. Enabling logging alone does not prove export.
4. Record a relative monotonic timeline, one action per capture, TX/RX direction
   and service/channel. Compare B4S against the same initial state and readback.
5. Export only minimal fixtures after manually removing addresses, serials,
   account identifiers, credentials and audio. Automated scrubbing cannot
   certify arbitrary vendor payloads anonymous.
6. Record per-feature hardware results separately from synthetic replay results.

## Export contract

Use `schemaVersion: 1`, `captureId` (U01 etc.), `identity` (model, firmware,
platform, transport, appVersion), `initialState`, `redaction` and `events`.
Set `redaction` to `reviewed-no-personal-data` only after reviewing the payload.
Events have ordered relative `atMs`, `kind` (tx/rx/action/disconnect/timeout),
uppercase contiguous `hex` for TX/RX and a human-written `note` for actions.
Do not check raw HCI files into Git. Missing identity evidence cannot close a gate.

## Current replay coverage

Synthetic vectors in `protocol/receiver.rs` cover every split point, batching,
CRC failure, bounded buffers, assembly timeout and reset. They do not establish
hardware support, command confirmation or transport choice. Bare AA frames
retain their GATT message boundary because they have no length field.

For acceptance, record actual behavior, state readback, ACK versus state reply,
external phone changes, reconnect and supported boundaries. Find/reset/OTA need
deliberate operator action and a documented stop/recovery path.
