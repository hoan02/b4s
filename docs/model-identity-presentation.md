# Model identity and presentation contract

All runtime profile filenames equal `<id>.json`. IDs are lowercase kebab-case,
derived from the exact model name after removing the Baseus brand prefix. `+`
becomes `plus`; distinct names with the same slug retain a deterministic hash
suffix. No source prefix (`server-`) is part of the live ID.

`scripts/sync-baseus-catalog.py` owns this naming rule for new public metadata.
The offline snapshot and runtime profiles use the same identities. Metadata
does not enable controls or select a protocol adapter. Saved device IDs and
custom EQ storage keys are migrated once under `b4s.migration.model-id.v2`;
existing canonical EQ data wins if both keys exist. Old IDs are accepted only
by the saved-data migration, never by live model resolution.

The APK 2.17.0.1 `AddDevicesListSecondFragment` assigns `model` to `deviceModel`,
`prodName` to `deviceName`, `icon` to `deviceIcon`, `iconLarge` to the large-image
field, and `colorList` to color metadata. B4S follows these data boundaries:

| Field | Meaning |
|---|---|
| `id` | Stable internal registry/storage key |
| `displayName` | Exact model identity for matching, independent of UI naming |
| `productName` | Public product name for scan and home presentation |
| `presentation` | Regional product name, category path, thumbnail/large URLs and color/image pairs |
| `imageUrl` | Effective product image: explicit profile asset, then large public art, then thumbnail |
| `imageProvenance` | Explicit profile or offline public metadata |

B4S selects regional metadata in US, EU, CN order. This region preference is a
desktop policy, not a claim that the APK merges regions the same way. Missing
names use the exact model identity. Generic `default_ear_pic` is a placeholder,
not a product override. Unavailable remote art uses the bundled placeholder.
Color entries describe available variants; no color is inferred from a name or
manufacturer bytes. The current device view uses the product image until a
device-color observation is implemented.

The original APK/decompiled sources and extraction caches remain local and
ignored. Only normalized public metadata and runtime profiles ship in the app.
