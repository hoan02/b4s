# Android 2.17.0.1 source triage

Input: completed local inventory report in ignored `.tmp/android-source-inventory.json`. This report provides marker locations rather than the original JADX execution error count. No decompiled code or native binaries are redistributed here.

## Headphone paths inspected

| Path | Marker findings | Decision |
| --- | --- | --- |
| EarEq classes | No error/undecompiled markers in scanned files | Consumer traces may inform source-derived EQ contracts; absence of markers does not prove correctness or firmware support |
| BluetoothDataWriteManager | No markers | Dispatch/framing contract documented separately; priority branch still needs caller/capture audit |
| HeadPhoneDataResolveManager | No markers | Wrapper source trace available; capture validation still required |
| classicbluetoothsdk | No markers | Socket lifecycle traced; stream framing/readiness/application callback coverage incomplete |
| DeviceManager | Undecompiled l(String), local line 2208 | Relevant blocker for source-derived ANC slider limits; verify DEX/smali or alternate engine |

`NoiseReducePopWindowV2.k`, local lines 150–151, calls DeviceManager.l(current model) and uses the returned pair for seek thresholds. The method body is skipped in this dump (310 instruction units). Do not treat the surrounding model lists or current B4S legacy maximum as reconstruction of this pair. Existing profile limits remain existing compatibility assumptions until model-specific source/capture evidence resolves them.

## Native dependency ownership

- APK inventory contains 106 native entries; entries across ABIs/splits are not 106 distinct libraries.
- `com.thingclips.ble.jni.BLEJniLib`, local line 74, loads BleLib. Its presence alone does not mean BP1 protocol framing requires a native port. A call graph from a supported headphone entrypoint is required.
- `com.baseus.networklib.utils.CommandFactory`, local line 24, loads native-lib and exposes nativeCreator/nativeParse; surrounding imports include non-headphone device request/response types. Do not reuse this generic network dependency as evidence for headphone commands.
- BES OTA packages include native D3DSdk/ImageMagic methods. OTA remains separate from the offline core scope; native names do not establish safe firmware/update support.

## Remaining triage work

1. Recover DeviceManager.l from the actual DEX with a second engine or smali and record model branches/ranges before changing ANC limits.
2. Extend call-path inventory to gestures, in-ear, multipoint, codec/hearing, SoundFit and firmware guards; prioritize marked methods actually reached by those entrypoints.
3. Index asset/resource configuration and reflection/JNI callers. Preserve package/version/tool/input provenance and original logs locally.
4. Record explicit unknowns per feature dossier. Marker-free source, successful compilation and metadata flags do not close hardware acceptance.
