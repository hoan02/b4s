# B4S

**Unofficial desktop companion** for multi-model earbuds listening control (Windows · macOS · Linux).

Scan nearby devices over Bluetooth LE, connect, and control common listening features — noise modes, EQ, spatial, game/low-latency, battery, find buds.

> **Not affiliated** with any earbud brand or their official apps. Trademarks belong to their owners.  
> Personal interoperability only.

---

## Why “B4S”?

Neutral product name — not a brand clone, not an “official desktop app”.  
Protocol support grows model-by-model; some devices are verified on hardware, others experimental.

---

## What it controls (listening)

- Bluetooth LE scan / connect / disconnect  
- Battery left · case · right (when the device notifies)  
- Noise: Normal · Ambient · ANC (+ strength where supported)  
- Spatial / panoramic (best-effort)  
- EQ presets (indices best-effort)  
- Game / low-latency mode  
- Find buds  

---

## Model support

| Level | Meaning |
|-------|---------|
| **Verified** | Packet table + control proven on real hardware |
| **Experimental** | Catalog match; tries BA/AA (+ 789C wrap when flagged) |
| **Scan only** | Name recognized; no control mapping yet |

Catalog: [`docs/protocol/models-catalog.md`](docs/protocol/models-catalog.md)

**Windows tip:** one pair often appears as **two scan entries** (BLE control vs audio). Use the entry that reaches a live control link.

---

## Stack

| Layer | Tech |
|-------|------|
| Shell | Tauri 2 |
| UI | SolidJS + SCSS |
| BLE | btleplug |
| Backend | Rust |

---

## Run

```bash
npm install
npm run tauri:dev
```

**Needs:** Node 18+, Rust stable, **Bluetooth ON**.

Demo UI (fake devices) is opt-in only when BT is off.

### Settings & auto-update

In-app **Cài đặt**: version, theme, check update.

```bash
npm run version:bump
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: release v…"
git tag v0.1.1 && git push origin main --tags
```

CI builds **Windows / macOS / Linux**. Details: [`docs/release.md`](docs/release.md).

Repo: [github.com/hoan02/b4s](https://github.com/hoan02/b4s)

---

## Layout

```
src/
  components/     HomePanel, MorePanel, Settings, BlePairing…
  lib/            BLE, device, theme, toast
src-tauri/src/
  ble.rs          Scan, connect, write, notify
  protocol/       Framing, wrap_v2, models catalog
docs/
  protocol/       Packet tables + model catalog
  re/             Local RE notes (bulk artifacts gitignored)
```

---

## Legal / hygiene

- Do **not** commit official APKs/XAPKs, private keys, or decompiled dumps  
- Do **not** present B4S as an official product  
- Device **marketing names** in the catalog are for BLE name matching only  

---

## License

MIT
