# BP1 Pro feature evidence matrix

Scope: Baseus 2.17.0.1 source observations and the B4S `bass-bp1-pro` profile.
This matrix separates source/replay implementation from physical-device
acceptance. The reviewed BP1 Ultra record remains scan-only and is outside this
control matrix because its transport has not been captured. The current repo
does not link a BP1 Pro capture ID or firmware manifest to these feature rows;
this matrix therefore records source/replay coverage separately from
firmware-specific acceptance.

| Feature | APK source trace | B4S behavior | Evidence result and remaining limit |
|---|---|---|---|
| ANC / transparency | `BA34` set and `AA34` state; mode and parameter are both significant | Implemented for BP1 Pro. Command waits for exact same-session mode+parameter observation. Snapshot v2 retains parameter/time; UI derives only profile-valid transparency submode and ANC environment/level. | **Implemented** at source/replay level. The checked-in dossier does not link a capture ID or firmware manifest; parameter coverage and freshness under external changes remain unverified per firmware. See [ANC dossier](bp1-pro-anc.md). |
| EQ preset / custom EQ | `BA30` query, `AA30` current index, `BA31` custom filter payload; model response includes `dictSort` and variable filter tuples | Implemented through the model EQ schema, exact index/readback and model-scoped custom draft. Empty server arrays do not grant or revoke a capability. | **Implemented** at source/replay level. Server snapshots and source are not device acceptance. See [EQ dossier](bp1-pro-eq-source.md). |
| Game mode | `BA23` query/state and `BA24` set | Implemented behind the BP1 Pro capability; write waits for matching game state. | **Implemented** at source/replay level; the checked-in dossier does not identify the capture/firmware scope for notification guards. |
| Bass boost | `BA53` query/state and `BA54` set | Implemented behind the BP1 Pro capability; write waits for matching observed level. | **Implemented** at source/replay level; per-firmware level semantics and hardware readback scope are not linked in the checked-in dossier. |
| Spatial audio | `BA42` enabled state, `BA43` selected mode, and `BA5E` support negotiation | Enable/disable is observed through `AA42`. The current state report does not confirm the selected mode; B4S keeps that UI selection unknown. | Enable control is **implemented** at source/replay level. Mode selection remains **unresolved** until a mode readback is identified and captured. |
| Find buds | `BA10` carries side and start/stop values | B4S sends start/stop only after its safety confirmation and reports transport acceptance. It does not claim sound started or stopped. | Request path is **implemented**. Actual playback, stop acknowledgement and cancellation behavior are **unresolved** pending hardware capture. |
| In-ear detection | `BA25` query and `BA26 00/01` set; `AA25` carries the switch; `AA26 0C/0D` are conflict errors | Backend encodes/decodes and validates; BP1 Pro exposes the command only under the Experimental opt-in, otherwise it is hidden. | **Implemented at source/replay level (experimental-only)**; wire values and guards are traced and covered by unit vectors in the [gesture/in-ear dossier](bp1-gesture-in-ear-source.md). Device readback authority, unsolicited updates, single-ear behavior and the model/firmware support list remain open. |
| Gesture mapping | `BA21`/`AA21` query-state and `BA22`/`AA22` set for v1, with `BA8C`–`BA8F`/`AA8C`–`AA8F` for v2 and `AA8B` support negotiation; bundled assets give model/function allowlists | Backend encodes/decodes v1 with reviewed layout/function validation; BP1 Pro exposes the control only under the Experimental opt-in, otherwise it is hidden. | **Implemented at source/replay level (experimental-only)**; layout bytes, per-side payload, function IDs, single/dual button guard and BP1 Pro allowlist are traced and tested ([dossier](bp1-gesture-in-ear-source.md)). Firmware choice between v1/v2, v2 child encoding, ACK semantics and hardware behavior remain open. |
| LDAC | `BA74` state query and `BA75` set; source call sites use differing polarity conventions | Capability is disabled in the BP1 Pro profile; no control is offered. | **Unresolved** for this model/firmware until polarity and required restart/reconnect behavior are reconciled. |
| Hearing protection | `BA94` is a write/ACK path; `AA93` carries state. See [hearing dossier](hearing-source.md). | Decoder evidence is retained, but profile capability and UI/write controls remain disabled. | **Planned** pending profile-specific level thresholds, initialization/readback guards and hardware evidence. |
| Multipoint (dual connection) | `BA57`/`AA57` query-state and `BA58`/`AA58` set-ack. See [multipoint dossier](bp1-multipoint-source.md). | Backend encodes/decodes and validates; BP1 Pro exposes the toggle only under the Experimental opt-in. | **Implemented at source/replay level (experimental-only)**. Interactions with in-ear/gesture and second-device behavior remain open. |
| Restore default settings | `BA36`/`AA36` availability and `BA37`/`AA37` restore (`00` success, `0C`/`0D` conflict). See [device-settings map](bp1-device-settings-source.md). | Backend encodes/decodes and validates; BP1 Pro exposes the action only under the Experimental opt-in behind a confirmation dialog. | **Implemented at source/replay level (experimental-only)**. Destructive behavior and post-restore state remain unverified. |

“Implemented” means the source-derived B4S command/state path is present; local
protocol and confirmation coverage varies by feature. It does not mean that a
BP1 Pro unit or any firmware has passed hardware acceptance. A public catalog
record, APK UI guard, or Android-side BLE command is not enough to promote a
feature to hardware-verified.
