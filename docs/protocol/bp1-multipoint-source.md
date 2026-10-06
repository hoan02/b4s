# BP1 multipoint source dossier

Scope: Baseus Android 2.17.0.1 decompiled source (local, gitignored). This dossier
records the dual-connection (multipoint) wire contract and its interaction with other
settings. It is an authored summary, not a copy of decompiled source.

Source pointers (local dump, not committed):

- `com/control_center/intelligent/view/viewmodel/EarHeadSetViewModel.java` (`T` = `BA57` query).
- `com/control_center/intelligent/view/presenter/HomeBleDataResolvePresenter.java` (each-connect query routing).
- `com/control_center/intelligent/view/fragment/ear/EarphoneFunctionShowFragmentNewUI.java` (`Setting.p`/`q` handlers, `Setting.n` setter, receive routing around `AA56`/`AA58`).

## 1. Wire contract

| Direction | Bytes | Meaning |
|---|---|---|
| Query | `BA57` | request the dual-connection switch |
| State | `AA57 <00\|01>` | `01` = multipoint on, `00` = off |
| Set | `BA58 <01\|00>` | `01` = enable, `00` = disable |
| Set reply | `AA58 <01\|00>` | `01` = applied, `00` = rejected |

The app's query handler (`onGetEachConnectSetQueryData`) reads the last byte as the
switch state. The set handler (`onGetEachConnectSettingData`) treats a trailing `01` as
success (it flips the UI switch) and a trailing `00` as failure (it shows the generic
"cannot set" toast and re-posts with `AA58 00`). So `AA58` is an acknowledgement, not an
independent state; `AA57` is the state.

## 2. Guards and interactions

- Visibility is driven by the model's switch-function list from the app dictionary, not
  by a dedicated `BA57` capability probe. B4S therefore needs a reviewed model capability
  rather than inferring support from a query.
- The same model setting is blocked in several places while the earbuds are already in
  dual-connection mode (`DeviceInfoModule.isDoubleEarConnect`): gesture configuration,
  in-ear detection and the low-frequency (`AA56`) setting refuse to write and surface a
  "double connection" message. Enabling multipoint can therefore make those controls
  temporarily unavailable, and a change from another controller can invalidate the
  current UI assumption.
- No reboot/reconnect is issued by the setter; the app only flips the switch after the
  reply.

## 3. B4S status and remaining unknowns

B4S implements `BA57`/`AA57` and `BA58`/`AA58` behind the reviewed `multipoint`
capability and the Experimental opt-in (source/replay level). Remaining unknowns:

- whether `AA57` is pushed unsolicited on external changes;
- the exact failure codes for `AA58` beyond `00`, and whether `01` is unconditional;
- whether a codec change or OS-level connection conflict is required;
- second-device identity and per-platform behavior (Windows audio routing vs the
  earbud's own paired-phone link);
- hardware acceptance on a specific firmware.
