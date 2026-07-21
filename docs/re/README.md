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

## What to extract for multi-model support

1. **Model names** → `protocol/models.rs` groups  
2. **Wire format flags** (bare BA vs 789C wrap, Classic BT vs BLE)  
3. **Listening opcodes** (noise, EQ, spatial, game, battery, find)  
4. **GATT UUID families** (BP1 custom, CCSDK `02F0…`, others)  

## Local artifacts (gitignored)

`docs/re/apk-*`, `tools/jadx`, `*.xapk` — large / copyrighted; regenerate as needed.
