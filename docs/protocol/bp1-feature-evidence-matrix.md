# BP1 Pro feature evidence matrix

Scope: Baseus 2.17.0.1 source observations and the B4S `bass-bp1-pro` profile.
This matrix separates source/replay implementation from physical-device
acceptance. The reviewed BP1 Ultra record remains scan-only and is outside this
control matrix because its transport has not been captured.

| Feature | APK source trace | B4S behavior | Evidence result and remaining limit |
|---|---|---|---|
| ANC / transparency | `BA34` set and `AA34` state; mode and parameter are both significant | Implemented for BP1 Pro. Command waits for exact same-session mode+parameter observation. Snapshot v2 retains parameter/time; UI derives only profile-valid transparency submode and ANC environment/level. | **Implemented** at source/replay level. No physical BP1 Pro capture; firmware-specific parameters and freshness under external changes remain unverified. See [ANC dossier](bp1-pro-anc.md). |
| EQ preset / custom EQ | `BA30` query, `AA30` current index, `BA31` custom filter payload; model response includes `dictSort` and variable filter tuples | Implemented through the model EQ schema, exact index/readback and model-scoped custom draft. Empty server arrays do not grant or revoke a capability. | **Implemented** at source/replay level. Server snapshots and source are not device acceptance. See [EQ dossier](bp1-pro-eq-source.md). |
| Game mode | `BA23` query/state and `BA24` set | Implemented behind the BP1 Pro capability; write waits for matching game state. | **Implemented** at source/replay level; hardware notifications and firmware guard still need capture. |
| Bass boost | `BA53` query/state and `BA54` set | Implemented behind the BP1 Pro capability; write waits for matching observed level. | **Implemented** at source/replay level; level semantics and hardware readback need capture. |
| Spatial audio | `BA42` enabled state, `BA43` selected mode, and `BA5E` support negotiation | Enable/disable is observed through `AA42`. The current state report does not confirm the selected mode; B4S keeps that UI selection unknown. | Enable control is **implemented** at source/replay level. Mode selection remains **unresolved** until a mode readback is identified and captured. |
| Find buds | `BA10` carries side and start/stop values | B4S sends start/stop only after its safety confirmation and reports transport acceptance. It does not claim sound started or stopped. | Request path is **implemented**. Actual playback, stop acknowledgement and cancellation behavior are **unresolved** pending hardware capture. |
| In-ear detection | `BA25` query and `BA26` set | No BP1 Pro capability or frontend control is exposed. | **Planned**; device readback, single-ear behavior and firmware constraints need tracing before implementation. |
| Gesture mapping | `BA21` query and `BA22` writes; bundled v1/v2 assets describe model-specific layouts/function IDs | No BP1 Pro capability or frontend control is exposed. Phone UI guards and persisted selections do not supply a verified B4S action mapping. | **Planned**; source guards, action-ID mapping, readback and hardware behavior remain open. |
| LDAC | `BA74` state query and `BA75` set; source call sites use differing polarity conventions | Capability is disabled in the BP1 Pro profile; no control is offered. | **Unresolved** for this model/firmware until polarity and required restart/reconnect behavior are reconciled. |
| Hearing protection | `BA94` is a write/ACK path; `AA93` carries state. See [hearing dossier](hearing-source.md). | Decoder evidence is retained, but profile capability and UI/write controls remain disabled. | **Planned** pending profile-specific level thresholds, initialization/readback guards and hardware evidence. |

“Implemented” means the source-derived B4S command/state path is present; local
protocol and confirmation coverage varies by feature. It does not mean that a
BP1 Pro unit or any firmware has passed hardware acceptance. A public catalog
record, APK UI guard, or Android-side BLE command is not enough to promote a
feature to hardware-verified.
