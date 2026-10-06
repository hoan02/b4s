# Model and protocol support

Runtime identities are resolved through exact names and aliases in reviewed
profiles plus the passive public catalog. The catalog is not a registry of
protocol guesses. A public product without an explicit control profile remains
scan-only with the `unknown` protocol family.

## Reviewed control profile

| Model | Profile | Family | Transport | Evidence boundary |
|---|---|---|---|---|
| Baseus Bass BP1 Pro | `bass-bp1-pro` | BP1 | BLE GATT, bare AA/BA | Reviewed UUID/framing and protocol behavior; firmware scope and current hardware acceptance remain tracked in the roadmap |

BP1 Ultra is recognizable but scan-only. Android source lists 789C wrapping
for this model, but the actual device firmware, transport path and Windows
connection have not been captured. Do not send a profile-independent query or
select an alternate GATT entry for it.

## Public catalog groups

The public snapshot contains product metadata used for scan presentation and
identity matching. Earbuds, over-ear, neckband and open-ear products may be
recognized; products classified as speakers are excluded from headphone
pairing. Recognition does not grant controls.

Examples of names in the public catalog include:

| Group | Examples |
|---|---|
| Bass BP1 / EP10 | BP1 Pro / Ultra / NC, EP10 Pro / Ultra / NC |
| Bowie MA | MA10, MA10s, MA20 |
| Bowie M | M2s, M3s, M4s, M2s Ultra |
| Bowie E / W / WM | E3, W04, WM01 |
| Open-ear | MC1 / MC2, AirGo, AS01 |
| Inspire | XP1, XH1, XC1 |
| Headsets and neckbands | H1 / H2, Max, P1, U2 |

These examples are not a compatibility promise. Product identity is matched
against explicit canonical names and aliases. No nearby edition inherits
another profile by substring or family resemblance.

## Support levels

| Level | Meaning |
|---|---|
| `verified` | Evidence supports the declared model and feature scope. |
| `experimental` | A profile exists, with remaining firmware/hardware evidence. |
| `scanOnly` | The app recognizes the name but does not enable control. |

See [how to add a model](../model-catalog.md) for profile and evidence
guidance. Images may use a generic presentation asset; image fallback never
affects identity or protocol routing.
