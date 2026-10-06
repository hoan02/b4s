# Protocol overview

Catalog recognition and working device control are separate. Public metadata
can identify a nearby product, but only an exact reviewed profile with an
explicit transport and capabilities can authorize control.

## Connection and protocol layers

1. Resolve the selected BLE advertisement through exact canonical names and aliases.
2. Load the reviewed profile and verify its declared transport and feature permissions.
3. Encode commands through that profile's explicit protocol family.
4. Apply only the profile's declared framing and GATT UUIDs.
5. Decode notifications through the same family and publish confirmed state.

BP1 Pro is the only reviewed active control profile. BP1 Ultra remains
scan-only while its firmware-specific transport is unresolved. Other catalog
products are passive until their own profile and evidence exist. See the
[model catalog](models-catalog.md) and [contribution guide](../model-catalog.md).

## BP1 Pro wire format

The logical commands use `BA`; device notifications use `AA`:

```text
App → device:  BA <command> <payload…>
Device → app:  AA <command> <payload…>
```

The reviewed BP1 Pro profile uses bare commands over the declared BLE GATT
service. A wrapped format observed in other model/firmware paths does not
authorize using it for BP1 Pro or BP1 Ultra. Framing is selected only by an
explicit model profile.

The BP1 Pro service and packet reference are documented in
[bp1-pro-anc.md](bp1-pro-anc.md). Other model families may use different UUIDs,
transports or command formats.

## Support levels

| Level | Meaning |
|---|---|
| `verified` | Hardware behavior is supported by evidence for the named model scope. |
| `experimental` | A reviewed profile exists, with incomplete model or firmware evidence. |
| `scanOnly` | The name can be recognized, but control is not enabled. |

Recognition, a successful GATT connection or a successful write alone does not
prove that a control is supported. Promote a model/feature only with hardware
evidence for its relevant firmware and transport.

## Reverse engineering

See the [research notes](../re/README.md) for the evidence workflow. Keep
proprietary APKs, firmware and decompiled source out of the repository; commit
only concise, independently useful protocol findings and reviewed sanitized
captures.
