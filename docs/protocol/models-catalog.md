# Model discovery registry

The registry in `src-tauri/src/protocol/models.rs` contains BLE name patterns
used to identify device models. It is primarily a discovery catalog: an entry
does not by itself mean the device has tested or enabled listening controls.

## Verified hardware targets

- Baseus Bass BP1 Pro
- Baseus Bass BP1 Ultra

Both use the BP1 protocol family, with model-specific framing and firmware
behavior. Consult the [BP1 packet reference](bp1-pro-anc.md) and the runtime
profile before changing commands.

## Example product groups in the registry

| Group | Examples |
|---|---|
| Bass BP1 / EP10 | BP1 Pro / Ultra / NC, EP10 Pro / Ultra / NC |
| Bowie MA | MA10, MA10s, MA20 |
| Bowie M | M2s, M3s, M4s, M2s Ultra |
| Bowie E / W / WM | E3, W04, WM01 |
| Open-ear | MC1 / MC2, AirGo, AS01 |
| Inspire | XP1, XH1, XC1 |
| Headsets and neckbands | H1 / H2, Max, P1, U2 |

These are registry examples, not a compatibility promise. Product names are
used to match BLE advertisements; aliases and support levels can change as
evidence is added.

## Support levels

| Level | Meaning |
|---|---|
| `verified` | Hardware behavior has been verified for the model. |
| `experimental` | A best-effort protocol mapping exists and needs more testing. |
| `scanOnly` | The app recognizes the name but does not enable control. |

See [how to add a model](../model-catalog.md) for profile and evidence guidance.
Product images are not bundled per model; the app uses a generic fallback image.
