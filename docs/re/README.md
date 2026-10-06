# Reverse-engineering notes (B4S)

Goal: expand **listening** control for many earbud lines (not only BP1-class).

> Personal interoperability only. Do **not** commit or redistribute third-party APK/XAPK dumps.

## Pipeline

```powershell
# 1) Unpack XAPK (zip)
# 2) jadx decompile base APK
# 3) python docs/re/scan_apk_strings.py
```

See [findings-2.14.1.md](./findings-2.14.1.md) for results used by B4S.
See [findings-2.17.0.1.md](./findings-2.17.0.1.md) for the newer APK analysis,
live public catalog endpoints, region coverage, and decoder corrections.

To refresh the public model metadata (CN, US and EU; no account required):

```powershell
python scripts/sync-baseus-catalog.py
```

The reviewed metadata snapshot is embedded for offline discovery. New models
remain recognition-only until their control protocol is configured and tested.

## What to extract for multi-model support

1. **Model names** → `protocol/models.rs` groups  
2. **Wire format flags** (bare BA vs 789C wrap, Classic BT vs BLE)  
3. **Listening opcodes** (noise, EQ, spatial, game, battery, find)  
4. **GATT UUID families** (BP1 custom, CCSDK `02F0…`, others)  

## Local artifacts (gitignored)

`docs/re/apk-*`, `tools/jadx`, `*.xapk` — large / copyrighted; regenerate as needed.


## Reproducible input and error inventory

```powershell
python scripts/inventory-android-source.py docs/re/apk-2.17.0.1
```

The report remains in ignored `.tmp/android-source-inventory.json`. It records SHA256/size of APK, XAPK, DEX and native inputs, hashes native and `res/`/`assets/` APK entries without extracting them, and locates JADX error/undecompiled-method markers without copying source. Resource records contain archive paths, sizes and hashes only; they do not inspect resource contents. Output is restricted to `.tmp`; do not commit the report or original dumps. Unpack XAPK splits before scanning. Marker counts are not the original JADX run's error count; retain original logs/tool versions and package/version provenance separately. Use the locations to triage headphone call paths, then verify failed methods against DEX/smali or another engine.
