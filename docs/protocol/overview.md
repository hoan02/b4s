# B4S — protocol overview (multi-model)

B4S is an **unofficial desktop listening companion** for multi-model earbuds, not a single-SKU tool.

Official mobile app (`com.baseus.intelligent`) supports many product lines. Desktop support is layered:

1. **Identify** BLE name → catalog entry (`protocol/models.rs`)  
2. **Pick wire format** — bare `BA`/`AA` vs **789C+CRC** wrap (Ultra / several “N0” models)  
3. **Send listening commands** — noise, EQ, spatial, game, battery query, find  
4. **Decode notifies** — battery, ANC ack, EQ, game state  

## Product lines in catalog

| Group | Examples |
|-------|----------|
| Bass BP1 / EP10 | BP1 Pro, BP1 Ultra, EP10 Pro/Ultra/NC |
| Bowie MA | MA10 / MA10s / MA20… |
| Bowie M | M2s, M3s, M4s, M2s Ultra… |
| Bowie E / W / WM | E3, W04, WM01… |
| Open-ear | MC1/MC2, AirGo, AS01… |
| Inspire | XP1, XH1, XC1 |
| Headset / neck | H1/H2, Max, P1, U2… |

See [models-catalog.md](./models-catalog.md).

## Wire formats

### Bare BA/AA (classic BP1 Pro captures)

```
App → device:  BA <cmd> <payload…>
Device → app:  AA <cmd> <payload…>
```

### 789C + CRC (official app `HeadPhoneDataResolveManager` for N0 models, e.g. BP1 Ultra)

```
78 9C | len_be16 | 02 | …payload… | crc16_be
```

Bare `BA…` is still the logical command; wrap is applied when the connected model needs it.

### GATT UUIDs commonly used

| Role | UUID |
|------|------|
| BP1-family service/write/notify | `53527aa4-…` / `ee684b1a-…` / `654b749c-…` |
| Bluetrum CCSDK fallback | `02f00000-…fe00` / `…ff01` / `…ff02` |

Not every Baseus model uses the same GATT. Check link health (write/notify UUIDs) after connect.

## Listening command map (logical)

| Feature | Logical TX | Notes |
|---------|------------|--------|
| Handshake | `BA 05 00` (+ `BA 05 01`) | After connect |
| Battery query | `BA 02` | Expect `AA 02` / case `AA 27` |
| Noise / ANC | `BA 34` mode + level | Off / ANC / Ambient |
| EQ / spatial payload | `BA 43` + byte | Shared opcode space in app |
| EQ query | `BA 42` | |
| Game / low latency | `BA 24` | Query `BA 23` |
| Find buds | `BA 10 02 01` | Both buds (app 2.14.1) |

Full BP1-oriented table: [bp1-pro-anc.md](./bp1-pro-anc.md).

## Support levels

| Level | Meaning for users |
|-------|-------------------|
| Verified | Works on tested hardware with known framing |
| Experimental | In catalog; best-effort BA/AA (+ 789C wrap) |
| Scan only | Recognized name only |

## Reverse engineering

Workflow and APK notes: [../re/README.md](../re/README.md).
