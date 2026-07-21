# Baseus Bass BP1 Pro ANC — Packet Table

> Source: reverse-engineered from official Baseus app + live BLE captures  
> (elaxptr/baseus-desktop, MIT). Integrated into this project.

## GATT endpoints

| Role   | UUID |
|--------|------|
| Service | `53527aa4-29f7-ae11-4e74-997334782568` |
| Write   | `ee684b1a-1e9b-ed3e-ee55-f894667e92ac` |
| Notify  | `654b749c-e37f-ae1f-ebab-40ca133e3690` |

## Frame format

```
Notify (device → app):  AA <cmd> <payload...>
Write  (app → device):  BA <cmd> <payload...>
```

No length field, no CRC on BLE path.

## Commands (app → device)

| Action | Bytes |
|--------|-------|
| ANC Off | `BA 34 00 FF` |
| ANC On (default strength) | `BA 34 01 68` |
| ANC On (custom 0–100%) | `BA 34 01 <level>` level = 0x10..0xFF |
| Transparency | `BA 34 02 FF` |
| EQ Balanced | `BA 43 00` |
| EQ Bass Boost | `BA 43 01` |
| EQ Voice | `BA 43 02` |
| EQ Clear | `BA 43 03` |
| EQ Query | `BA 42` |
| Game Mode ON | `BA 24 01` |
| Game Mode OFF | `BA 24 00` |
| Find Buds (candidate) | `BA 92 01` |

## Notifications (device → app)

| Event | Bytes | Notes |
|-------|-------|-------|
| Battery L/R | `AA 02 <L%> 00 <R%> 01` | 0% = bud in case |
| Case battery | `AA 27 <case%> <charging>` | charging: 00/01 |
| ANC Off ack | `AA 34 00` | |
| ANC ack (flat) | `AA 34 01` | some firmwares always send this |
| ANC active | `AA 33 …` | |
| Transparency | `AA 32 …` | |
| EQ ack | `AA 43 <preset>` | |
| EQ query resp | `AA 42 <preset>` | |
| Game state | `AA 23 <00\|01>` | real state |
| Game ack | `AA 24 01` | ignore (no state) |
| Identity | `AA 12 <mac…>` | on connect |
| Case event | `AA 80 …` | partially decoded |
| Keepalive | `AA 30 …` | ignore |

## Implementation

- Rust: `src-tauri/src/protocol/`
- Wired in: `src-tauri/src/ble.rs` (subscribe + write)
- Frontend events: `device://battery`, `device://anc`, `device://eq`, `device://game`
