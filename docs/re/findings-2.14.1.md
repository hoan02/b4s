# Findings — protocol notes (for B4S)

Used by **B4S** to support **many audio lines**, not only BP1-class devices.

Source: `com.baseus.intelligent_2.14.1.xapk`  
Tools: DEX string parse + **jadx 1.5.1** decompile.

## Listening command map (official app)

Confirmed in decompiled sources under `com.control_center.intelligent`:

| Feature | TX (hex string form) | Source |
|---------|----------------------|--------|
| Handshake | `BA0500` | `HomeBleDataResolvePresenter` |
| Handshake alt | `BA0501` | Anti-lost / some accessories |
| ANC set | `BA34` + mode(2) + level(2/`FF`) | `BleCommandUtil.k` |
| EQ set classic | `BA43` + `00`… | `EarphoneFunctionShowFragmentNewUI` |
| EQ set (codec guards) | `BA43` + `0A`/`0B`/`0C`/`0D` | LHDC/LDAC/dual/call guards |
| EQ query | `BA42` | EQ activities |
| EQ custom reset | `BA4300` | `EarEqSelfDefineActivity` |
| Game mode set | `BA24` + payload | `GestureBleManager.g` |
| Game mode query | `BA23` | `GestureBleManager.e` |
| Find both start | `BA100201` | `EarphoneFunctionShowFragmentNewUI` |
| Find left start | `BA100001` | same |
| Find right start | `BA100101` | same |
| Find stop both/L/R | `BA100200` / `BA100000` / `BA100100` | same |
| In-ear detect | `BA25` query / `BA26` set | `GestureBleManager` |
| Gesture map | `BA21` query / `BA22` set | `GestureBleManager` |

### ANC mode bytes (unchanged)

- Off: mode `00`, level `FF`
- ANC: mode `01`, level `10`–`FF`
- Transparency: mode `02`, level often `FF`

### Classic EQ presets (listening UI)

| Byte | Preset |
|------|--------|
| `00` | Balanced / classic |
| `01` | Bass |
| `02` | Voice |
| `03` | Clear (legacy) |

Newer models may use `0A+` for spatial/codec-linked modes — desktop sticks to classic 00–03 for broad compatibility.

## GATT

| Role | UUID |
|------|------|
| Service (BP1 family) | `53527AA4-29F7-AE11-4E74-997334782568` |
| Write / Notify | same as prior BP1 Pro captures (`ee684b1a…` / `654b749c…`) — not plain-string in DEX |

Other families in app (not desktop priority): Bluetrum `02F0…`, short `ae`/`fae`/`fd`.

## Model catalog (listening only)

- **~124 models** extracted from DEX product strings (TWS / open-ear / headset / neck).
- Excluded: chargers (PPS/BPM), mice (F02), non-audio.
- Groups for UI: Bass BP1/EP10, MA, M, E, Open-ear, Inspire, Bass line, W/WM, AirNora, Eli, Headset, Neckband, AeQur…
- Generated into `src-tauri/src/protocol/models.rs`
- **Verified** (hardware packet table): Bass BP1 Pro, Bass BP1 Ultra  
- Rest: **Experimental** — try BP1 BA/AA protocol (best-effort).

## Critical: BP1 Ultra ≠ bare BA over BLE only

From `DeviceManager` + `BluetoothDataWriteManager` (app 2.14.1):

| Flag | BP1 Pro | BP1 Ultra |
|------|---------|-----------|
| `E0` → **Classic BT (SPP)** path | **true** | **true** |
| `N0` → **789C + CRC wrap** | false | **true** |

So official app for **BP1 Ultra**:
1. Wraps every `BA…` command as `789C | len | 02 | … | CRC16`
2. Prefers **Classic Bluetooth** (`ClassBt`) not pure BLE for writes

Bare `BA 34 01 68` on BLE (what desktop sent before) is **ignored** by Ultra.

Desktop now:
- Detects Ultra (and other N0 models) → auto **wrap_v2**
- Handshake tries wrapped `BA0500` first
- Still on **BLE GATT** (Windows); if still no notify, SPP may be required next

## Desktop wiring (this update)

1. Full listening model registry + `group` field  
2. Handshake `BA0500` / wrap_v2 / `BA0501`  
3. Post-connect: `BA42` + `BA23`  
4. Find buds: `BA100201`  
5. CRC framing module `protocol/wrap_v2.rs` (tests green)  
6. Link health UI  


## UI strings / features (app 2.14.1)

| Feature | App evidence | Desktop |
|---------|--------------|---------|
| Noise Normal / Ambient / ANC | `noise_normal_mode_*`, `ambient`, `noise_reduction_*` | UI + BA34 |
| Spatial on + Music / Cinema | `panoramic_sound_*`, SpatialSoundManger codes 00–05; set `BA43`+mode | UI + BA43 |
| Spatial query | `BA42` / `BA5E00\|01` | partial |
| EQ list | server dict + labels (Classic, Bass, Jazz, Classical, …) | 11 chips, BA43+index |
| SoundFit | layout `layout_sound_fit`, entry on ear screen | UI entry only |
| Game / low latency | `BA24` | toggle |
| LDAC / hearing protection | `str_ldac*`, `str_hearing_protection*` | UI only (codec often OS-level) |
| Battery | query **`BA02`** (wrap 789C if `DeviceManager.N0`), notify **`AA02LL00RR01`** via `BleUtils.d` → `"L-R"`; case **`AA27`** (`substring(4,6)` = %) | BA02 bare+789C; unwrap_v2 matches HeadPhoneDataResolveManager; salvage AA02 in raw |

Product photos: not in APK as per-model packs (CDN). Generic `default_ear_pic.png` only.

## Out of scope (for now)

- OTA / firmware  
- Full SoundFit hearing test flow  
- LDAC codec switch on Windows  
- Gesture remapping  
- Custom EQ curves / AI tuning  
