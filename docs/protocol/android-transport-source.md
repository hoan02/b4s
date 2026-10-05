# Android 2.17.0.1 transport source inventory

This is a manually interpreted transport contract plus an exact-name guard inventory from the local dump. It does not certify Windows transport support, firmware applicability or model capabilities. APK/decompiled source remains ignored.

## Dispatch entry points

| Source class / method | Role | Observed contract |
| --- | --- | --- |
| BluetoothDataWriteManager.Companion.a | Connect dispatch | Valid Bluetooth address and enabled adapter; H0 chooses ClassBt.g, otherwise BleApi.G |
| Companion.b | Framing | Q0 wraps through HeadPhoneDataResolveManager.f unless the supplied command already starts with 789C |
| Companion.c | Normal write | Wrapped result goes to ClassBt.b for H0, otherwise Ble.b |
| Companion.d | Priority write | Classic sends wrapped result; BLE priority branch passes original sourceData to Ble.o. This divergence needs a caller/capture audit |
| Companion.e | Alternate write | H0 chooses ClassBt.j; otherwise Ble.k, both using framing result |
| ClassicBtConnectManager | Android socket | createRfcommSocketToServiceRecord receives its constructor UUID; do not infer a fixed SPP UUID from the socket API |
| HeadPhoneDataResolveManager.Companion.f | Wrapper builder | Adds 789C length/header around command data; branch-specific layouts require receiver/capture parity |

Pointers in the local dump: DeviceManager.H0 line 291, Q0 line 808; BluetoothDataWriteManager.Companion methods a–e; ClassicBtConnectManager line 163; HeadPhoneDataResolveManager.f line 500. Line numbers refer to this dump, not B4S source.

## Exact model guards

The lists are independent. Absence from one guard only selects the Android default branch; it never grants a reviewed B4S connection descriptor. Names must not be matched by substring or inherited across variants.

| Android exact model name | H0 transport dispatch | Q0 framing dispatch |
| --- | --- | --- |
| Baseus AS01 | Classic | Unwrapped branch |
| Baseus AS01 Air | Classic | 789C |
| Baseus BH1 NC Lite | Classic | 789C |
| Baseus Bass 1+ | Classic | Unwrapped branch |
| Baseus Bass BC1 | Classic | Unwrapped branch |
| Baseus Bass BC1 Lite | Classic | 789C |
| Baseus Bass BC1 星光版 | Classic | 789C |
| Baseus Bass BC2 | Classic | 789C |
| Baseus Bass BC2s | Classic | 789C |
| Baseus Bass BD1 | Classic | Unwrapped branch |
| Baseus Bass BF1 | Classic | Unwrapped branch |
| Baseus Bass BF1 Lite | Classic | 789C |
| Baseus Bass BH1 | Classic | Unwrapped branch |
| Baseus Bass BH1 Air | Classic | Unwrapped branch |
| Baseus Bass BH1 Lite | Classic | Unwrapped branch |
| Baseus Bass BH1 NC | Classic | Unwrapped branch |
| Baseus Bass BP1 NC | Classic | 789C |
| Baseus Bass BP1 Pro | Classic | Unwrapped branch |
| Baseus Bass BP1 Ultra | Classic | 789C |
| Baseus Bass BS1 | Classic | Unwrapped branch |
| Baseus Bass BS1 Lite | Classic | Unwrapped branch |
| Baseus Bass BS1 NC | Classic | Unwrapped branch |
| Baseus Bass BS2 Lite | Classic | Unwrapped branch |
| Baseus Bass BS2 NC | Classic | 789C |
| Baseus Bass E19s | Classic | Unwrapped branch |
| Baseus Bass EH10 NC | Classic | Unwrapped branch |
| Baseus Bass EP10 NC | Classic | 789C |
| Baseus Bass EP10 Pro | Classic | Unwrapped branch |
| Baseus Bass EP10 Ultra | Classic | 789C |
| Baseus Bass GH03 | Classic | 789C |
| Baseus Bass TWS 1 | Classic | Unwrapped branch |
| Baseus Bass W04 | Classic | 789C |
| Baseus Bass WM01s | Classic | 789C |
| Baseus Bass WM02s | Classic | 789C |
| Baseus Bowie E3 2025 | Classic | Unwrapped branch |
| Baseus Bowie M3s | Classic | Unwrapped branch |
| Baseus Bowie M4s | Classic | 789C |
| Baseus Bowie MA10s | Classic | Unwrapped branch |
| Baseus Bowie MC1 | Classic | Unwrapped branch |
| Baseus Bowie MC1 Pro | Classic | Unwrapped branch |
| Baseus Bowie MC2 | Classic | 789C |
| Baseus Bowie MC2 Air | Classic | Unwrapped branch |
| Baseus Bowie MC2 NC | Classic | 789C |
| Baseus Bowie MC2 S | Classic | Unwrapped branch |
| Baseus Bowie MC2 S 先锋版 | Classic | Unwrapped branch |
| Baseus Bowie MF1 | Classic | Unwrapped branch |
| Baseus Bowie MH1 | Classic | Unwrapped branch |
| Baseus Bowie MP1 | Classic | 789C |
| Baseus Bowie MS1 | Classic | 789C |
| Baseus Bowie Test | Classic | Unwrapped branch |
| Baseus EH10 NC Lite | Classic | 789C |
| Baseus Eli 15i Fit | Classic | Unwrapped branch |
| Baseus Eli Sport2 | Classic | Unwrapped branch |
| Baseus Inspire XC1 | Classic | 789C |
| Baseus Inspire XH1 | Classic | 789C |
| Baseus Inspire XP1 | Classic | 789C |
| Baseus Sleep SK1 | Classic | Unwrapped branch |

## Remaining gates

- Trace constructor UUID source, socket read/write ownership, callback routing and startup readiness before Windows transport implementation.
- Triage relevant JADX errors using DEX/smali or a second engine before trusting incomplete branches.
- Record package/tool/input hashes and ARM64/native dependencies separately; this inventory covers dispatch guards only.
- U01 must establish actual BP1 Ultra transport, endpoint/UUID, framing and firmware. The runtime remains scan-only until that evidence and Windows acceptance exist.
- Priority-path framing divergence requires actual caller coverage; do not silently reproduce or repair an Android behavior without evidence.
