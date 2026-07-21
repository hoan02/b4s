# Baseus Desktop App

Unofficial desktop client for Baseus earbuds (Windows / macOS / Linux).

Built with **Tauri 2 + SolidJS + SCSS**. Protocol reverse-engineered from the official Baseus app + live BLE captures (BP1 Pro ANC).

> ⚠️ Not affiliated with Baseus. All trademarks belong to their respective owners.

---

## Features

- ✅ **Bluetooth LE** scan / pair / disconnect
- ✅ **Baseus protocol (BP1 Pro ANC)** — real GATT commands
  - Battery L / R / Case (live notify)
  - ANC Off / On / Transparency + strength
  - EQ presets (Balanced / Bass / Voice / Clear)
  - Game / low-latency mode
  - Find My Buds
- ✅ Demo mode when no Bluetooth adapter
- ✅ Dark UI matching official Baseus app


## Supported models

| Level | Models |
|-------|--------|
| ✅ **Verified** | Bass BP1 Pro ANC only |
| 🟡 **Experimental** (~70) | Full official-app TWS/headset list (MA10, M2s, E3, WM01, Encok, Inspire…) — tries BP1 protocol |
| ⚪ Scan only | Unknown Baseus-like names |

See [`docs/protocol/models-catalog.md`](docs/protocol/models-catalog.md).

Use **Capture Studio** to verify Experimental models on your hardware.

## Protocol (BP1 Pro ANC)

| | UUID |
|---|---|
| Service | `53527aa4-29f7-ae11-4e74-997334782568` |
| Write | `ee684b1a-1e9b-ed3e-ee55-f894667e92ac` |
| Notify | `654b749c-e37f-ae1f-ebab-40ca133e3690` |

```
Write  : BA <cmd> <payload>
Notify : AA <cmd> <payload>
```

| Action | Packet |
|--------|--------|
| ANC On | `BA 34 01 68` |
| ANC Off | `BA 34 00 FF` |
| Transparency | `BA 34 02 FF` |
| EQ Bass | `BA 43 01` |
| Game ON | `BA 24 01` |
| Battery notify | `AA 02 <L> 00 <R> 01` |
| Case notify | `AA 27 <%> <chg>` |

Full table: [`docs/protocol/bp1-pro-anc.md`](docs/protocol/bp1-pro-anc.md)

## Tech Stack

| Layer | Choice |
|-------|--------|
| Framework | Tauri 2 |
| Frontend | SolidJS + SCSS |
| BLE | btleplug 0.11 |
| Protocol | Custom (framing + BP1 Pro) |
| Language | TypeScript + Rust |

## Run

```bash
npm install
npm run tauri:dev
```

**Requirements:** Node 18+, Rust stable, Bluetooth ON  
(Linux: `bluez` + user in `bluetooth` group)

## Project layout

```
src/
  components/     BlePairing, Battery, AncControl, EqCards…
  lib/ble.ts      BLE API
  lib/device.ts   Protocol commands + live events
src-tauri/src/
  ble.rs          Scan / connect / subscribe / write
  protocol/       Framing + BP1 Pro decoder + command builders
    framing.rs
    types.rs
    bp1_pro.rs
docs/protocol/    Packet tables
```

## Events (frontend)

| Event | Payload |
|-------|---------|
| `device://battery` | `{ left, right, case, *Charging }` |
| `device://anc` | `off` \| `anc` \| `transparency` |
| `device://eq` | preset name |
| `device://game` | boolean |
| `ble://scan-status` | devices list |
| `ble://connected` | BleDevice |

## Capture Studio

In-app reverse-engineering toolkit (button 🔬 in titlebar):

1. **Start capture** while connected
2. Follow **Guided steps** (ANC, EQ, Game…)
3. Watch live **Hex log** (TX orange / RX cyan) with auto-decode hints
4. Inspect **GATT map** discovered on connect
5. **Raw write** arbitrary hex to probe opcodes
6. **Export** JSON bundle or Markdown packet table

Backend: `src-tauri/src/capture.rs`  
Frontend: `src/components/CaptureStudio.tsx`

## Roadmap

- [x] BLE pair
- [x] BP1 Pro protocol (ANC / EQ / Game / Battery)
- [x] Capture Studio (hex log, guided, export)
- [ ] More models (use Capture Studio → export bundle)
- [ ] Auto-reconnect last device
- [ ] System tray + low-battery notification
- [ ] Custom EQ bands
- [ ] Gesture config

## Credits

Protocol data adapted from [elaxptr/baseus-desktop](https://github.com/elaxptr/baseus-desktop) (MIT).

## License

MIT
