# Baseus model catalog

Source: Official Baseus app (`com.baseus.intelligent`) supported device list + hardware-verified BP1 Pro protocol.

## Support levels

| Level | Meaning |
|-------|---------|
| **Verified** | Packet table + GATT confirmed on real hardware |
| **Experimental** | In official app list; app tries BP1-compatible protocol (may fail if UUID/framing differs) |
| **ScanOnly** | Name looks like Baseus but not in catalog mapping |

## Verified (full control)

- Baseus Bass BP1 Pro ANC (`Bass BP1 Pro`)

## Experimental TWS (try BP1 protocol)

Bowie MA10 / MA10s / MA10 Pro / MA20 / MA20 Pro  
Bass BD1 / Bass 1+  
Bowie M3, M2s Ultra, M2s Pro, M2s, M2+, M2, M1  
Bowie E13, E12, E10, E8, E5, E5x, E3, E2, EX, Baseus E9  
Bowie W04 family, WM01–WM05, WX5, MZ10, EZ10  
Bowie MC1/MC2, MF1, AirGo AS01 / 1 Ring / AG20  
Eli Sport 1, Eli Fit, Eli 10i Fit  
AirNora / 2 / 3, Bowie 30 / 35, AeQur G10  
Storm 1/3, C-Mic CM10  
Encok W04/W04 Pro/W05 Lite/W11/W12  
Inspire XP1 / XH1 / XC1  

## Experimental headset / neck

Bowie 30 Max, 10 Max, H2, H1 Pro, H1s, H1, D05, AeQur GH02  
P1 Lite, P1x, P1, Bowie U2 Pro, U2  

## How to promote Experimental → Verified

1. Connect device in app  
2. Open **Capture Studio** → Start capture  
3. Run Guided steps (ANC, EQ, Game, Battery)  
4. Export Markdown/JSON  
5. Add `docs/protocol/<model>.md` + decoder module if framing differs  

## Notes

- Many Baseus TWS use Bluetrum chips; some share similar control framing with BP1 Pro, others use different GATT UUIDs (e.g. classic `02F0…` CCSDK).  
- Experimental models are **best-effort** — if control fails, use Capture Studio and contribute captures.  
