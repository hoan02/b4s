# Architecture and extension boundaries

B4S is a SolidJS frontend calling a Tauri/Rust backend. Maintain stable Tauri
commands and events while improving the internal implementation.

| Area | Responsibility | Entry point |
| --- | --- | --- |
| Frontend | Screens, local state, Tauri bridge | `src/App.tsx`, `src/lib/ble.ts`, `src/lib/device.ts` |
| Model catalog | Model-specific aliases, capabilities, ANC and EQ data | `src-tauri/catalog/models/*.json` |
| Catalog loader | Embed profiles, parse typed data, validate invariants | `src-tauri/build.rs`, `src-tauri/src/catalog/` |
| Device registry | Resolve discovery or an explicit model to runtime configuration | `src-tauri/src/device/registry.rs` |
| Initialization | Plan startup queries from capabilities | `src-tauri/src/device/initialization.rs` |
| Protocol router | Select command encoding and reply decoding by family | `src-tauri/src/protocol/router.rs` |
| Protocol family | Wire commands and notification semantics | `src-tauri/src/protocol/families/` |
| BLE transport | Discovery, connection, GATT subscription, writes, live events | `src-tauri/src/ble.rs`, `src-tauri/src/ble/` |

## Profile loading

The build script enumerates JSON files in filename order and embeds them in the
binary. Adding a profile does not require updating a Rust list. Profiles are
parsed once and validated before use; duplicate IDs, unregistered families,
invalid support levels, misspelled capabilities and inconsistent EQ curves fail
validation. `cargo test --lib` exercises validation without Bluetooth.

The runtime model registry overlays JSON profiles onto the legacy discovery
catalog. A JSON profile replaces the entry with the same ID. Its capabilities and
noise settings take precedence. This lets contributors migrate one model at a
time without removing existing experimental discovery entries.

## Protocol routing

BP1 code lives in `protocol/families/bp1.rs`. The router selects both writes and
decoded replies. Experimental Baseus AA/BA devices still reuse the existing BP1
decoder as a best-effort compatibility path. Unknown families produce raw events
rather than interpreting replies as BP1 state. The existing generic battery
recovery path remains in the BLE layer.

## Remaining migration work

This is an incremental structure, not a claim that all devices are supported:

- Legacy discovery entries still live in `protocol/models.rs`; migrate them only
  with model-specific evidence instead of generating speculative profiles.
- GATT defaults, handshake and 789C wrapping still have existing shared logic.
  Models needing different transport behavior require changes and tests there.
- Common AA/BA encoders remain in `protocol/mod.rs`; move new family-specific
  wire behavior into the family implementation and route it explicitly.
- Some frontend EQ and listening controls still use shared constants. Adding
  different band counts or controls requires wiring profile data through those
  screens and the command boundary, not merely editing JSON.

Prefer completing one model's discovery-to-control path over declaring support
for a large catalog at once.
