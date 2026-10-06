# Android 2.17.0.1 source triage

Input: completed local inventory report in ignored `.tmp/android-source-inventory.json`. This report provides marker locations rather than the original JADX execution error count. No decompiled code or native binaries are redistributed here.

## Headphone paths inspected

| Path | Marker findings | Decision |
| --- | --- | --- |
| EarEq classes | No error/undecompiled markers in scanned files | Consumer traces may inform source-derived EQ contracts; absence of markers does not prove correctness or firmware support |
| BluetoothDataWriteManager | No markers | Dispatch/framing contract documented separately; priority branch still needs caller/capture audit |
| HeadPhoneDataResolveManager | No markers | Wrapper source trace available; capture validation still required |
| classicbluetoothsdk | No markers | Socket lifecycle traced; stream framing/readiness/application callback coverage incomplete |
| DeviceManager | Undecompiled l(String), local line 2208 | Recovered with JADX raw-instruction fallback; see range contract below |

`NoiseReducePopWindowV2.k`, local lines 150–151, calls DeviceManager.l(current model) and uses the returned pair for seek thresholds. The normal method body is skipped (310 instruction units). A targeted raw-instruction fallback recovered its exact-name branches; the interpreted range contract below replaces the earlier source gap. Device acceptance still needs captures.

## Native dependency ownership

- APK inventory contains 106 native entries; entries across ABIs/splits are not 106 distinct libraries.
- `com.thingclips.ble.jni.BLEJniLib`, local line 74, loads BleLib. Its presence alone does not mean BP1 protocol framing requires a native port. A call graph from a supported headphone entrypoint is required.
- `com.baseus.networklib.utils.CommandFactory`, local line 24, loads native-lib and exposes nativeCreator/nativeParse; surrounding imports include non-headphone device request/response types. Do not reuse this generic network dependency as evidence for headphone commands.
- BES OTA packages include native D3DSdk/ImageMagic methods. OTA remains separate from the offline core scope; native names do not establish safe firmware/update support.

## Remaining triage work

1. Cross-check the recovered range branches with model captures; investigate any conflicting firmware behavior before changing runtime limits.
2. Extend call-path inventory to gestures, in-ear, multipoint, codec/hearing, SoundFit and firmware guards; prioritize marked methods actually reached by those entrypoints.
3. Resource archive paths, sizes and hashes are now inventoried locally by `inventory-android-source.py`; inspect likely headphone assets/configuration and reflection/JNI callers without copying binaries or source into Git. Preserve package/version/tool/input provenance and original logs locally.
4. Record explicit unknowns per feature dossier. Marker-free source, successful compilation and metadata flags do not close hardware acceptance.


## Recovered ANC slider range contract

Tool: local JADX 1.5.6, one class only, `--decompilation-mode fallback --comments-level debug`. Run completed successfully against the existing base APK. Raw output remains ignored at `.tmp/device-manager-fallback.java`; no third-party code is copied into this document. This is raw-instruction inspection using the same engine, not a claimed independent decompiler result.

The initial minimum is 1. Exact-name branches target one of three return sites:

| Return site | Minimum / maximum | Model scope |
| --- | --- | --- |
| Laa | 1 / 3 | BH1 NC Lite, EH10 NC Lite, Bowie MC2 NC, Bass BS2 NC |
| Ld0 | 1 / 5 | Bass BP1 Pro, BP1 Ultra, EP10 Pro, EP10 Ultra, BS1 NC, BH1 NC, EH10 NC; Bowie M3s, M4s, MH1, MP1, MS1; Inspire XC1, XH1, XP1 |
| Ldb | 1 / 10 | Null/unmatched Android model name |

Branch equality checks precede each return. BP1 Pro and Ultra both jump to Ld0. The existing BP1 Pro profile's maxCustomLevel=5 therefore agrees with this source; no runtime range expansion is needed. The Android fallback 10 must not authorize an unknown B4S model. This pair describes UI slider thresholds, not wire opcode, firmware support, runtime readiness or hardware acceptance.
