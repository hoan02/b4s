# Reference packet table — Bass BP1 Pro / Ultra family

> Reference implementation for the **best-documented** Baseus listening path.  
> **B4S** uses this table for verified BP1-class devices and as a template for other lines.  
> Other Baseus models may share BA/AA opcodes, use **789C+CRC**, or different GATT UUIDs.

Sources: live BLE captures + official app analysis (elaxptr/baseus-desktop + APK 2.14.1).

## Device notes

| App product name | Wire notes |
|------------------|------------|
| Bass BP1 Pro | Often bare `BA`/`AA` on BLE |
| Bass BP1 Ultra | Official app often wraps with **789C+CRC**; may prefer Classic BT in app |

## GATT (BP1-family custom service)

| Role | UUID |
|------|------|
| Service | `53527aa4-29f7-ae11-4e74-997334782568` |
| Write | `ee684b1a-1e9b-ed3e-ee55-f894667e92ac` |
| Notify | `654b749c-e37f-ae1f-ebab-40ca133e3690` |

## Frame format (bare)

```
Notify (device → app):  AA <cmd> <payload...>
Write  (app → device):  BA <cmd> <payload...>
```

Ultra / N0 models: logical BA command wrapped as `789C | len | … | CRC` (see `protocol/wrap_v2.rs`).

## Commands (logical BA)

| Action | Bytes |
|--------|-------|
| Handshake | `BA 05 00` (fallback `BA 05 01`) |
| Battery query | `BA 02` |
| ANC Off | `BA 34 00 FF` |
| ANC On | `BA 34 01 <level>` |
| Transparency | `BA 34 02 FF` |
| EQ / spatial payload | `BA 43 <byte>` |
| EQ query | `BA 42` |
| Game ON/OFF | `BA 24 01` / `BA 24 00` |
| Game query | `BA 23` |
| Find both buds | `BA 10 02 01` |
| Find L / R | `BA 10 00 01` / `BA 10 01 01` |

## Notifications (logical AA)

| Event | Bytes |
|-------|-------|
| Battery L/R | `AA 02 <L%> 00 <R%> 01` |
| Case battery | `AA 27 <case%> <charging>` |
| ANC ack | `AA 34 …` (firmware-dependent) |
| EQ ack | `AA 43 <preset>` |
| Game state | `AA 23 <00\|01>` |
| Identity | `AA 12 …` |

## Implementation (B4S)

- Rust: `src-tauri/src/protocol/`  
- BLE: `src-tauri/src/ble.rs`  
- Multi-model overview: [overview.md](./overview.md)  
