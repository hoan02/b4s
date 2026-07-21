# Model catalog

The catalog follows the same separation used by the APK: model data selects a
protocol family, while the family adapter owns packet encoding. A model profile
must not contain BLE packet-building code.

## Add a model with an existing family

1. Add `src-tauri/catalog/models/<model-id>.json`.
2. Put EQ presets and custom-band limits in the profile's `eq` object.
3. Put ANC environments and limits in `noise`.
4. Add an image reference under `image` or extend the image resolver with a
   local asset.
5. Set `support` to `experimental` until real hardware captures verify every
   command.

The profile's `protocolFamily` must be an existing family such as `bp1`. The
router then reuses that family adapter without changing the UI or model data.

## Add a new protocol family

Create a new adapter under `src-tauri/src/protocol/families/`, register the
family in the router, and add frame tests for query, write, and notification
commands. Do not mark the profile `verified` until those tests are backed by
hardware captures.

The current hardware test target is Baseus Bass BP1 Ultra. It belongs to the
`bp1` protocol family, but it must remain a separate model profile from BP1 Pro
because framing and firmware behavior can differ. The existing profile is the
first extracted BP1 profile:
`src-tauri/catalog/models/bass-bp1-pro.json`.
