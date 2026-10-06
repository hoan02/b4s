# Hearing protection Android 2.17.0.1 source dossier

Source evidence only; BP1 Pro's reviewed profile does not enable hearing protection. This dossier does not grant support to other models or firmware.

| Path | Local pointer | Contract |
| --- | --- | --- |
| EarSoundSettingsActivity.X1 | 578–594 | Selected DbValueBean enabled/value is sent through the viewmodel; six-second UI timeout |
| EarSoundSettingViewModel threshold list | 134 | UI offers dB values 75, 80, 85, 90, 95, 100 |
| EarSoundSettingViewModel.U | 479–480 | BA94 + enabled 00/01 + value hex; -1 maps to FF |
| EarPodNewActivity | 2007–2010 | Alternate path can send enabled + FF or selected value |
| EarSoundSettingsActivity AA93 branch | 371 onward | Query-state consumer; region/language and DeviceManager.R guard can suppress UI adoption |
| EarSoundSettingsActivity AA94 branch | 388 onward | Result 01 is ACK; failure handled separately, not threshold state |
| HearingHealthActivity | 420 | BA93 query |

Source contract: AA93 carries the current enabled flag and raw threshold; AA94 carries ACK/error results and is never device state. The earlier B4S setter guard 0–3/default 1 did not match this consumer contract. Per-model threshold/sentinel constraints are now required before hearing can be enabled. Do not interpret FF as 255 dB.

Remaining work: trace DeviceManager.R and all model/firmware/region guards; determine meaning of -1/FF during readback and whether toggling preserves the current threshold; define profile thresholds/default/sentinel policy; capture each selectable threshold and error path; provide an editor that displays threshold and confirmed state. Snapshot presence does not prove capability or safe threshold application.

Implementation: the 0–3/default-1 assumptions are replaced by explicit per-model thresholds and FF permission. Toggle uses the observed raw threshold; missing observations reject. AA94 is not decoded as confirmed state. No current model is newly enabled by this schema change.
