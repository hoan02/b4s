# Model catalog

The offline catalog also embeds `src-tauri/catalog/baseus-public.json`,
a public metadata snapshot merged across Baseus CN/US/EU category APIs. Run
`python scripts/sync-baseus-catalog.py` to refresh it explicitly. It records
server identities, categories, regional image URLs and color codes; it does
not supply protocol capabilities. New headphone identities remain `scanOnly`
with an `unknown` protocol and cannot initiate a control connection. Pairing
filters catalog products whose category paths in every region identify them as
speakers; unclassified or regionally mixed products stay visible. CDN images
are not fetched automatically. Reviewed profiles remain separate from this
public metadata snapshot. See [2.17.0.1 findings](re/findings-2.17.0.1.md) for
scope and completeness limits.

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

Current family values are `bp1` and `unknown`. Support values are `verified`,
`experimental`, and `scanOnly`. For recognition-only, use `scanOnly` and
`unknown`. A JSON profile supplies explicit runtime transport, ANC settings and
capabilities; no registry fallback or transport default authorizes control. See
[architecture](architecture.md) for the profile boundary.

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

BP1 Pro has a reviewed JSON profile at
`src-tauri/catalog/models/bass-bp1-pro.json`. BP1 Ultra is recognizable but
remains scan-only while its control transport is unverified. Keep their model
records separate because framing and firmware behavior can differ. Do not infer
that the JSON catalog is complete from public metadata or a shared family name. The app's `verified`
support label is reserved for model and feature behavior checked on real
hardware; consult the progress tracker for remaining evidence gates.
