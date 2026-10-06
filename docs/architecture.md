# Architecture and extension boundaries

B4S is a SolidJS frontend calling a Tauri/Rust backend. Product data, protocol
permissions, BLE transport and confirmed device state have separate owners.
Changes to a frontend/backend contract are coordinated in the same change and
versioned when the payload shape changes.

| Area | Responsibility | Entry point |
| --- | --- | --- |
| Frontend shell | Navigation, app lifecycle and composition | `src/App.tsx` |
| Frontend runtime state | Session ordering, device subscriptions and feature operations | `src/stores/`, `src/features/` |
| Tauri adapters | Typed command/event DTOs and contract validation | `src/lib/ble.ts`, `src/bridge/deviceSnapshot.ts` |
| Model profiles | Reviewed transport, capabilities, ANC and EQ constraints | `src-tauri/catalog/models/*.json` |
| Public catalog | Passive identity and display metadata for discovery | `src-tauri/catalog/baseus-public.json` |
| Catalog loader | Embed profiles, parse types and validate invariants | `src-tauri/build.rs`, `src-tauri/src/catalog/` |
| Device registry | Exact identity resolution and explicit profile selection | `src-tauri/src/device/registry.rs` |
| Capability and initialization | Authorize operations and plan declared startup queries | `src-tauri/src/device/capability.rs`, `initialization.rs` |
| BLE runtime | Mutable session, discovery, connection, commands and GATT transport | `src-tauri/src/ble/runtime.rs`, `src-tauri/src/ble/` |
| Protocol router | Select an explicit family codec for commands and replies | `src-tauri/src/protocol/router.rs`, `families/` |
| Device state | Confirmed observations with session and revision identity | `src-tauri/src/device/snapshot.rs` |

## Profile and identity rules

The build script enumerates JSON profiles in filename order and embeds them in
the binary. Profiles are parsed once and validated before use; duplicate IDs,
unregistered families, invalid support levels, misspelled capabilities and
inconsistent EQ curves fail validation.

Reviewed profiles are the only source of control permissions. Public catalog
metadata can identify and display a scan result, but it remains scan-only until
a reviewed profile explicitly declares a supported transport and capabilities.
Identity resolution uses exact canonical names and aliases. It does not infer
capabilities, framing or edition from substrings. Stored identifiers can be
updated only through a deliberate, versioned, one-time data migration; that
mapping is never used for live device routing.

## Protocol and transport rules

The BP1 codec lives in `protocol/families/bp1.rs`. The router dispatches both
command encoding and reply decoding from the selected profile's family. Unknown
families cannot inherit BP1 decoding or a generic AA/BA adapter. A new family
needs its own codec and evidence-backed profile before it can control hardware.

Transport is model-profile driven. The connection path uses the selected OS
device entry, declared service/write/notify UUIDs, handshake and notification
framing. It does not probe generic characteristics, switch to a same-name
sibling, or fall back to a different frame format. BLE discovery, connection,
command handling and GATT I/O are separate modules; shared mutable state is
owned by `ble/runtime.rs`.

Device snapshots are the source of confirmed feature state. Write completion
alone is not readback. Snapshot session IDs and revisions prevent stale updates
from replacing observations after disconnect or reconnect. Scan, connection
and link-health DTOs each carry a contract version; frontend adapters reject
unsupported versions rather than interpreting them as the current shape.

## Extending a model or family

For an existing family, add a JSON profile only when the model identity,
transport, firmware scope and feature constraints are supported by source or
capture evidence. A profile alone does not prove the hardware works. Keep a
model scan-only when those facts are unresolved; do not inherit a nearby model's
capabilities by similarity.

For a new protocol family, add a codec under
`src-tauri/src/protocol/families/`, register the family in catalog validation,
and add vectors for commands, replies, malformed frames and readback. Keep its
transport independent from feature encoding. Hardware verification is recorded
per feature, model, firmware, transport and platform; synthetic replay does not
promote support to verified.

Prefer completing one model's discovery-to-control path over declaring support
for a large catalog at once. Current evidence and remaining gates are tracked
in [the implementation roadmap](headphone-desktop-progress.md).

## Localization

English is the first-launch default and fallback. The five shipped locales are
bundled in the frontend and registered in `src/lib/i18n.ts`; visible UI strings
belong in `src/locales/<locale>/translation.json`. The locale checker runs in CI.
See the [translation guide](translations.md) before adding or changing strings.
