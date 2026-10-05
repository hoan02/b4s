# Protocol overview

B4S is structured for multiple models, but catalog recognition and working
device control are separate. Most names in the discovery registry are not
verified hardware targets. Use the support level in the model registry and the
app before relying on a command.

## Connection and protocol layers

1. Match a BLE advertisement to a model entry in `src-tauri/src/protocol/models.rs`.
2. Resolve its capabilities, protocol family and transport settings.
3. Encode a logical command through the selected family adapter.
4. Apply model-specific framing and send it over the discovered GATT link.
5. Decode notifications and update the device state exposed to the frontend.

The current verified hardware targets are Baseus Bass BP1 Pro and BP1 Ultra.
Other Baseus entries may share protocol hints or names without verified control.
See the [registry overview](models-catalog.md) and [model contribution guide](../model-catalog.md).

## BP1-family wire format

The logical commands use `BA`; device notifications use `AA`:

```text
App → device:  BA <command> <payload…>
Device → app:  AA <command> <payload…>
```

BP1 Pro uses the bare command format. BP1 Ultra uses the `789C` wrapper with
length and CRC for the applicable commands. Framing is selected by the device
profile; do not assume all models in the family use the same transport details.

The BP1 custom GATT service and packet reference are documented in
[bp1-pro-anc.md](bp1-pro-anc.md). Other model families may use different UUIDs,
transports or command formats.

## Support levels

| Level | Meaning |
|---|---|
| `verified` | The model is explicitly identified as a hardware-tested target. |
| `experimental` | A best-effort profile exists; model or firmware behavior is not fully verified. |
| `scanOnly` | The name can be recognized, but control is not enabled. |

Recognition, a successful GATT connection or a successful write alone does not
prove that a control is supported. Promote a model only with hardware evidence
for the relevant behavior.

## Reverse engineering

See the [research notes](../re/README.md) for the evidence workflow. Keep
proprietary APKs, firmware and decompiled source out of the repository; commit
only concise, independently useful protocol findings and sanitized captures.
