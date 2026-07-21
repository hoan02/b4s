# B4S — listening model catalog

**B4S** targets multi-model audio earbuds (TWS, open-ear, headset, neckband), not a single SKU.

Source: public product strings + hardware checks. Names below are for BLE advertisement matching only.

Excluded: chargers, mice, scent/purifier, non-audio.

## Support levels

| Level | Meaning |
|-------|---------|
| **Verified** | Packet table + control proven on real hardware |
| **Experimental** | Catalog match; B4S tries BA/AA (+ 789C wrap when flagged) |
| **Scan only** | Name looks like Baseus; no control mapping yet |

## Verified (listening control)

- Baseus Bass **BP1 Ultra** (often 789C+CRC wrap)  
- Baseus Bass **BP1 Pro** (often bare BA/AA)  

## Groups (~120+ experimental entries)

| Group | Examples |
|-------|----------|
| Bass BP1 / EP10 | BP1 Ultra/Pro/NC, EP10 Ultra/Pro/NC |
| Bowie MA series | MA10 / MA10s / MA10 Pro / MA20 / MA20 Pro |
| Bowie M series | M1–M4s, M2s Pro, M2s Ultra |
| Bowie E series | E3/E5/E10/E12/E13, E9, EX |
| Open-ear | MC1/MC2/MF1, AirGo AS01/AG20 |
| Inspire | XP1 / XH1 / XC1 |
| Bass line | BD1, BC1/2, BF1, BH1, BS1/2, Bass 1+ |
| Bowie W / WM | W04 family, WM01–05, WX5, MZ10, EZ10 |
| AirNora | AirNora / 2 / 3 |
| Eli sport | Eli Sport, Eli Fit… |
| Headset | H1/H2, 10/30/35 Max, D05, MH1 |
| Neckband | P1 / P1x / P1 Lite, U2 |
| AeQur | G10, GH02, N10… |

Registry code: `src-tauri/src/protocol/models.rs`.

## Promote Experimental → Verified

1. Connect in **B4S** (Bluetooth ON)  
2. Confirm **Live** link (notifies + battery when possible)  
3. Exercise noise / EQ / game / battery; note AA notifies in logs
4. Document differences in `docs/protocol/`  

## Product images

Product photos are not bundled per model; B4S uses a generic ear asset.
