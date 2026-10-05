# Model catalog

The catalog follows the same separation used by the APK: model data selects a
protocol family, while the family adapter owns packet encoding. A model profile
must not contain BLE packet-building code.

## Add a model with an existing family

1. Add `src-tauri/catalog/models/<model-id>.json`.
   Profiles are embedded automatically at build time; no Rust list registration
   is needed. Use `bass-bp1-pro.json` as a shape reference, replacing its values
   with evidence for the new model rather than assuming the same capabilities.
2. Put EQ presets and custom-band limits in the profile's `eq` object.
3. Put ANC environments and limits in `noise`.
4. Add an image reference under `image` or extend the image resolver with a
   local asset.
5. Set `support` to `experimental` until real hardware captures verify every
   command.

The profile's `protocolFamily` must be an existing family such as `bp1`. The
router then reuses that family adapter. A profile alone does not implement new
wire behavior or guarantee that every UI control works on the model.

Currently registered family values are `bp1`, `baseusAaBaExperimental`, and `unknown`.
Support values are `verified`, `experimental`, and `scanOnly`. For recognition
only, use `scanOnly` and `unknown`. JSON profiles override matching legacy IDs
and supply runtime ANC settings and capabilities. Transport defaults and some
frontend controls still need migration; see [architecture](architecture.md).

Run `cargo test --manifest-path src-tauri/Cargo.toml --lib` to validate all
profiles. EQ curves must match the number of bands, gains need valid limits,
and preset `dictSort` values must be unique within the model. BP1 custom EQ still
requires eight bands on the wire; allowing other band counts in catalog data
does not make them supported by that adapter or the current UI.

## Add a new protocol family

Create a new adapter under `src-tauri/src/protocol/families/`, register the
family in the router, and add frame tests for query, write, and notification
commands. Do not mark the profile `verified` until those tests are backed by
hardware captures.

Register the JSON family name in catalog validation and runtime model resolution
as well. Route reply decoding alongside writes, and review startup queries,
handshake, UUID selection and wrapping for the new family.

Baseus Bass BP1 Pro and BP1 Ultra are the verified hardware targets for the
`bp1` family. They remain separate model records because framing and firmware
behavior can differ. Only BP1 Pro currently has an extracted JSON profile at
`src-tauri/catalog/models/bass-bp1-pro.json`; BP1 Ultra and legacy entries are
still represented by the discovery registry. Do not infer that the JSON catalog
is complete from the verified status of a registry entry.
