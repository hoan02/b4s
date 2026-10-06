# Headphone model support matrix

Generated from `src-tauri/catalog/baseus-public.json` (fetched 2026-10-05T17:05:01.049502+00:00) and reviewed JSON profiles in `src-tauri/catalog/models/` by `scripts/generate-headphone-support-matrix.py`.

The public snapshot has 174 products, of which 129 are audio products. Category data consistently classifies 5 audio products as speakers; the remaining 124 identities are candidates for headphone discovery. Recognition is metadata only. `scanOnly` and `unknown` mean no feature can be controlled.

The matrix records profile permissions, not per-feature hardware acceptance. The BP1 Pro profile currently enables the listed features, while its firmware scope and release acceptance remain tracked in [`headphone-desktop-progress.md`](headphone-desktop-progress.md). BP1 Ultra remains scan-only pending captured transport/firmware evidence. Every public candidate without a reviewed profile is scan-only with no inferred feature support.

| Catalog ID | Product | Catalog group | Support | Family | Profile-enabled features | Evidence / limit |
|---|---|---|---|---|---|---|
| `server-aequr-g10` | AeQur G10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-aequr-gh02` | AeQur GH02 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airgo-1-ring` | AirGo 1 Ring | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airgo-ag20` | AirGo AG20 | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airgo-as01` | AirGo AS01 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airnora-2` | AirNora 2 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airnora-3` | AirNora 3 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-airnora` | Baseus AirNora | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-as01` | Baseus AS01 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-as01-air` | Baseus AS01 Air | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-1-plus` | Baseus Bass 1+ | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bc1-b0f249a2` | Baseus Bass BC1 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bc1-lite` | Baseus Bass BC1 Lite | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bc1-ec048a8e` | Baseus Bass BC1 星光版 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bc2` | Baseus Bass BC2 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bc2s` | Baseus Bass BC2s | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bd1` | Baseus Bass BD1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bf1` | Baseus Bass BF1 | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bf1-lite` | Baseus Bass BF1 Lite | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bh1` | Baseus Bass BH1 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bh1-air` | Baseus Bass BH1 Air | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bh1-lite` | Baseus Bass BH1 Lite | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bh1-nc` | Baseus Bass BH1 NC | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bp1-nc` | Baseus Bass BP1 NC | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bp1-pro` | Baseus Bass BP1 Pro | tws | `verified` | `bp1` | ANC/transparency, EQ presets, Custom EQ, Game mode, Bass boost, Spatial, Find buds | docs/protocol/bp1-pro-anc.md; firmware scope not yet captured |
| `server-bass-bp1-ultra` | Baseus Bass BP1 Ultra | tws | `scanOnly` | `unknown` | — | docs/re/findings-2.17.0.1.md; 789C source candidate; U01 transport capture required |
| `server-bass-bs1` | Baseus Bass BS1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bs1-nc` | Baseus Bass BS1 NC | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-bs2-lite` | Baseus Bass BS2 Lite | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-e19s` | Baseus Bass E19s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-eh10-nc` | Baseus Bass EH10 NC | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-ep10-nc` | Baseus Bass EP10 NC | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-ep10-pro` | Baseus Bass EP10 Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-ep10-ultra` | Baseus Bass EP10 Ultra | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-w04` | Baseus Bass W04 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-wm01s` | Baseus Bass WM01s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-wm02s` | Baseus Bass WM02s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bh1-nc-lite` | Baseus BH1 NC Lite | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e3-2025` | Baseus Bowie E3 2025 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m3s` | Baseus Bowie M3s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m4s` | Baseus Bowie M4s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ma10s` | Baseus Bowie MA10s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc1` | Baseus Bowie MC1 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc1-pro` | Baseus Bowie MC1 Pro | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc2` | Baseus Bowie MC2 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc2-air` | Baseus Bowie MC2 Air | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc2-nc` | Baseus Bowie MC2 NC | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc2-s-713196ac` | Baseus Bowie MC2 S | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mc2-s-ef41b558` | Baseus Bowie MC2 S 先锋版 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mf1` | Baseus Bowie MF1 | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mh1` | Baseus Bowie MH1 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mp1` | Baseus Bowie MP1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ms1` | Baseus Bowie MS1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowiee2` | Baseus BowieE2 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowiee3` | Baseus BowieE3 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowiee8` | Baseus BowieE8 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-e9` | Baseus E9 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eh10-nc-lite` | Baseus EH10 NC Lite | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-15i-fit` | Baseus Eli 15i Fit | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-sport2` | Baseus Eli Sport2 | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-ex` | Baseus EX | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-inspire-xc1` | Baseus Inspire XC1 | Ear clip type | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-inspire-xh1` | Baseus Inspire XH1 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-inspire-xp1` | Baseus Inspire XP1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-p1` | Baseus P1 | Neck Hanging Series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-p1-lite` | Baseus P1 Lite | Neck Hanging Series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-p1x` | Baseus P1x | Neck Hanging Series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-storm-1` | Baseus Storm 1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-storm-3` | Baseus Storm 3 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-w04-pro` | Baseus W04 Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-w04-plus-pro` | Baseus W04+Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-w05lite` | Baseus W05Lite | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-wm02` | Baseus WM02 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-wm02-plus` | Baseus WM02+ | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-baseusencokw04` | BaseusEncokW04 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-baseusencokw11` | BaseusEncokW11 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-baseusencokw12` | BaseusEncokW12 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bass-e12x` | Bass E12x | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1s-be5ff335` | Bowie  H1S | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-10-max` | Bowie 10 Max | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-30` | Bowie 30 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-30-max` | Bowie 30 Max | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-35` | Bowie 35 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-d05` | Bowie D05 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e10` | Bowie E10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e12` | Bowie E12 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e13` | Bowie E13 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e5` | Bowie E5 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-e5x` | Bowie E5x | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ez10` | Bowie EZ10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1` | Bowie H1 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1-pro` | Bowie H1 Pro | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1i` | Bowie H1i | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1s-e19e2d01` | Bowie H1s | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h1s-pro` | Bowie H1s Pro | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-h2` | Bowie H2 | Headwear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m1` | Bowie M1 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m2` | Bowie M2 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m2-plus` | Bowie M2+ | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m2s` | Bowie M2s | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m2s-pro` | Bowie M2s Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-m3` | Bowie M3 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ma10` | Bowie MA10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ma10-pro` | Bowie MA10 Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ma20` | Bowie MA20 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-ma20-pro` | Bowie MA20 Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-mz10` | Bowie MZ10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-u2` | Bowie U2 | Neck Hanging Series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-u2-pro` | Bowie U2 Pro | Neck Hanging Series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-w04` | Bowie W04 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-w04-plus-13faaa6c` | Bowie W04 Plus | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-w04-pro` | Bowie W04 Pro | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-w04-plus-2153e749` | Bowie W04+ | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-wm01` | Bowie WM01 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-wm01plus` | Bowie WM01Plus | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-wm03` | Bowie WM03 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-wm05` | Bowie WM05 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-bowie-wx5` | Bowie WX5 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-c-mic-cm10` | C-Mic CM10 | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-10i-fit` | Eli 10i Fit | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-1i-fit` | Eli 1i Fit | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-fit` | Eli Fit | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-eli-sport-1` | Eli Sport 1 | Open ended | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |
| `server-m2s-ultra` | M2s Ultra | In ear series | `scanOnly` | `unknown` | — | No reviewed control profile; recognition metadata only. |

## Speaker products excluded from headphone pairing

| Catalog ID | Product | Classification |
|---|---|---|
| `server-aequr-30-air` | AeQur 30 Air | Excluded: speaker-only category in every region. |
| `server-aequr-ds10` | AeQur DS10 | Excluded: speaker-only category in every region. |
| `server-aequr-n10` | AeQur N10 | Excluded: speaker-only category in every region. |
| `server-aequr-vo20` | AeQur VO20 | Excluded: speaker-only category in every region. |
| `server-sleep-sk1` | Baseus Sleep SK1 | Excluded: speaker-only category in every region. |

This matrix is refreshed when the public snapshot or a reviewed profile changes. It does not replace feature/model/firmware/platform hardware reports, and synthetic replay does not promote support to verified.
