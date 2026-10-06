# Baseus 2.17.0.1: public catalog and protocol findings

Inspected locally on 2026-10-05/06 for B4S interoperability. This document
contains reviewed observations, not third-party decompiled source.

## Source and extraction

- XAPK: `Baseus_2.17.0.1.xapk`, 303,324,957 bytes.
- SHA-256: `a4ffc52b65f489914093fbe6c8eb8aa2e6e896b31ca4dbb024565962f3326222`.
- Package `com.baseus.intelligent`, version name `2.17.0.1`, version code `184`.
- Main APK plus `config.arm64_v8a.apk` and `config.mdpi.apk`.
- Local JADX 1.5.6 run with `--no-res`: 42,724 classes, 508 reported errors.
  Source output was produced, but this is not a claim that every method was
  recovered correctly. Native libraries and resources were not fully analyzed.
- DEX string-table scan: 180 literal BA command candidates, 84 UUID candidates,
  190 model/name candidates. These include other device families and SDKs;
  they are not an earbud support matrix or a complete server catalog.
- Local output is under gitignored `docs/re/apk-2.17.0.1/`. APKs, dumps and
  tool binaries must not be committed or bundled with B4S.

## Catalog comes from the server

Static evidence in `com.base.baseus.api.ControlApi`:

| Endpoint | Method and parameters | Purpose |
|---|---|---|
| `/app/category` | GET, `version=1`; optional `fqa` for the FAQ flow | Add-device category tree |
| `/app/category/getModelParams` | GET, `model`; some call sites also supply `color`, `version`, `alarm` | Model-specific EQ/media/configuration |
| `/app/category/getModelSkuList` | GET, `model` | SKU metadata |
| `/app/devicebind/getProductResource` | POST, JSON `ResRequest` | Product resource lookup; not exercised |
| `/app/homepage/dictByName` | GET, `dictName`, optional `dictDetailNames`, `model` | Feature dictionaries; not enumerated |

`AddDevicesListActivity.mCategoryVersion` is `1`; its add-device path calls
`ControlServices.j1(version)`. There are no page/size arguments in that API
signature. The response is a recursive tree, not just a flat model list:

`data[] -> category.child[] -> category.products[]`

Product records contain `model`, `prodName`, `icon`, `iconLarge`, `type`,
`categoryId`, and nullable `colorList` (`color`, `url`). Category type `2`
is audio, including speakers; category IDs vary by region. Deduplicate using
the full `model` string, not display labels, SKU/color, or category ID.

`NetWorkApi` identifies the CN, US, and EU hosts. `OkHttpCacheInterceptor`
adds `platform: 1`, `lang`, `appVersion`, `versionCode`, and account headers
when applicable. Bare category requests returned HTTP 200 with application
error `Unsupported platforms`. Supplying the normal Android platform/version
headers returned `code: 0` without any account/auth/anonymous token.

### Live read-only results

Fetched with `version=1`, `lang=en`, `platform=1`, `appVersion=2.17.0.1`,
`versionCode=184`; no login, binding, device serial, or private keys.

| Region | Host | Products | Audio |
|---|---|---:|---:|
| CN | `bds-api-cn.baseus.cn` | 151 | 115 |
| US | `bds-api-us.baseus.com` | 142 | 115 |
| EU | `bds-api-eu.baseus.com` | 142 | 115 |
| Union by full model | All three | 174 | 129 |

This is all products **published in these category responses** at fetch time.
It does not establish all historical/hidden/test products, firmware variants,
or every language/account-dependent result. The APK includes additional
hardcoded model names and aliases, so the public snapshot can omit older
products. B4S intentionally does not restore those names as an implicit runtime
registry: an older identity without a reviewed profile remains unsupported.

### B4S implementation

`scripts/sync-baseus-catalog.py` fetches all selected regions and writes a
deterministic model ordering to `src-tauri/catalog/baseus-public.json`. The
snapshot includes provenance, region/category variants, color codes, and
HTTPS image references. It allowlists metadata fields, rejects API/schema
errors, and only replaces the previous snapshot after all regions succeed.
It can replay local raw responses with `--input-dir` for reproducibility.

B4S embeds audio discovery records from this snapshot. Reviewed JSON profiles
are the only source of control permissions; new full identities get
`scanOnly`, protocol `unknown`, no capabilities and no guessed GATT UUIDs.
Identity resolution uses exact canonical names and aliases. Connection follows
the selected OS entry and profile UUIDs; unknown protocol does not probe GATT or
inherit another model's adapter. Non-audio products remain metadata and are not
registered as earbuds.

CDN URLs are retained for an eventual explicit metadata/image provider.
Discovery does not fetch images or call Baseus servers at runtime; the current
offline placeholder remains. The synchronizer is a developer operation.

## EQ is model-specific too

Read-only US `getModelParams` probes succeeded for both BP1 targets:

- `Baseus Bass BP1 Pro`: seven `eq_sound_mode` entries, `dictSort` values
  `0, 1, 3, 7, 8, 9, 10`: Baseus Classic, Deep Bass, Hi-Fi Live, Jazz,
  Classical, Treble Boost, Acoustic.
- `Baseus Bass BP1 Ultra`: an empty `eq_sound_mode` array for the same query.

Descriptions encode a prefix followed by variable-frequency filter tuples,
not an eight-band fixed-grid gain curve. The existing BP1 profile contains
12 presets with display curves; it must not be described as the current
server response. Empty data is not proof a device has no EQ: the app can use
other dictionaries, model/firmware conditions and local fallbacks. Do not
automatically convert these records into command capabilities or replace
reviewed EQ behavior without tracing the relevant consumers.

## Listening commands traced in 2.17.0.1

| Feature | Evidence | Observation |
|---|---|---|
| ANC | `BleCommandUtil.Companion`, noise builder | `BA34 + mode + level/FF` retained |
| Game | `GestureBleManager.e/g` | Query `BA23`, set `BA24 + flag` |
| In-ear detection | `GestureBleManager.a/b` | Query `BA25`, set `BA26 + flag` |
| Gesture mapping | `GestureBleManager.d/f` | `BA21` query and `BA22` writes; event/side mappings vary |
| Find buds | `EarphoneFunctionShowFragmentNewUI` | `BA10 + 00(left)/01(right)/02(both) + 01(start)/00(stop)` |
| Spatial | `PanoramicSoundViewModel.E/G/H` | Query `BA42`, support negotiation `BA5E + flag`, selection `BA43 + mode` |
| Bass | `EarSoundSettingViewModel.L/N` | Query `BA53`, set `BA54`; compact and enable/level forms exist |
| LDAC | `LdacSettingActivity.m1/onEvent` | Query `BA74`, activity writes `BA75 00` for enabled, `01` for disabled |
| Hearing protection | `EarSoundSettingViewModel.U`, `HearingProtectionPopWindow.z` | Set `BA94 + enabled + level/FF`; state reply `AA93 + enabled + level` |

The ear fragment uses another LDAC boolean convention in some call sites;
do not generalize the activity's polarity to every model/firmware.
Likewise spatial and EQ share an opcode, and many controls depend on model
guards in `DeviceManager`. These observations do not verify each function on
all server-listed audio models.

### Concrete decoder correction

B4S accepts only the two-byte-minimum `AA93` hearing state. `AA94` is a write
acknowledgement/error path (see `HeadPhoneMainActivity.ResultHandle.E`), not
device state; every AA94 payload is excluded from state confirmation. Short and
invalid frames cannot overwrite the snapshot. This is covered by offline
decoder tests; no hardware validation was performed in this run.

## Reproduce

```powershell
# After extracting the XAPK locally:
./tools/jadx-1.5.6/bin/jadx.bat --no-res -d docs/re/apk-2.17.0.1/jadx-out docs/re/apk-2.17.0.1/extracted/com.baseus.intelligent.apk
python docs/re/scan_apk_strings.py --apk docs/re/apk-2.17.0.1/extracted/com.baseus.intelligent.apk --out docs/re/apk-2.17.0.1/strings

# Refresh published catalog metadata explicitly (no account required):
python scripts/sync-baseus-catalog.py

# Offline parser and protocol validation:
python scripts/test-sync-baseus-catalog.py
python docs/re/test_scan_apk_strings.py
cargo test --manifest-path src-tauri/Cargo.toml --lib
npx tsc --noEmit
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Next protocol work should select a physical model, trace its guards/transport
and EQ dictionary consumers, then verify request/reply captures. Catalog
completeness and functional device support are separate validation tasks.
