# BP1 device settings source map

Scope: Baseus Android 2.17.0.1 decompiled source (local, gitignored). This is an
authored map of the device-settings opcodes that were confidently identified, plus the
ones still unresolved. It does not claim hardware acceptance.

Source pointers (local dump, not committed):

- `com/control_center/intelligent/view/fragment/EarphoneSettingFragment.java` and its `$mReveiver$1`.
- `com/control_center/intelligent/view/activity/headsetting/EarPhoneSettingV2Activity.java`.
- `com/control_center/intelligent/view/activity/headphones/QuickCallSettingActivity.java`.
- `com/control_center/intelligent/view/activity/CallNumSettingActivity.java`.
- `com/control_center/intelligent/view/presenter/HomeBleDataResolvePresenter.java`.

## Confident mappings

| Feature | Host write | Device report | Meaning |
|---|---|---|---|
| Firmware version | `BA19` | `AA19 …` | version query; the receive path logs "earphone version" and forwards the raw frame. Payload parsing not yet traced. |
| Restore-defaults availability | `BA36` | `AA36 …` | query; `BleCommandUtil.e` decides whether the restore row is shown. |
| Restore default operation settings | `BA37` | `AA37 <code>` | `00` = applied, `0C` = blocked by dual connection, `0D` = blocked during a call. |
| Call feature availability | `BA44` | `AA44 …` | query; controls call-related row visibility. |
| Call-number query | `BA45` | `AA45 …` | reply length ≥ 6. |
| Call-number state | `BA46` | `AA46 <text…>` | the text after the opcode is the stored call number. |
| Set call number | `BA47` + encoded number | `AA47 <code>` | acknowledgement. |
| Touch-control lock | `BA4A01` / `BA4A00` | `AA4A …` | `01` disables touch controls, `00` enables; the receiving path at `EarPhoneSettingV2Activity` reads `AA4A`. |

The restore and call settings share the same conflict vocabulary already seen elsewhere:
`0C` = dual connection active, `0D` = in a call.

## Desktop applicability

- Firmware version, touch-control lock and restore defaults are desktop-relevant
  candidates.
- Call number / quick call is a phone-side feature (the earbud stores a number to dial);
  on Windows it is at most a candidate and may be not applicable, consistent with the
  roadmap's "Remote camera/call" row.

## Still unresolved

- `BA19` payload encoding (ASCII vs BCD vs structured) and whether it is pushed at
  startup.
- The auto power-off ("`time_off`") opcode; `BA46`/`BA4A` candidates were ruled out and
  no confident match was found.
- `BA36` availability semantics and whether it is a boolean or a bitfield.
- `BA3F`/`BA4A` effect on audio (the label is confirmed but the runtime behavior is not).
- `BA49`, `BA55`, `BA90`, `BA9A`, `BA56`, `BA77`, `BA70`–`BA7D` remain unlabeled.
- Prompt language/volume and indicator-light opcodes were not located in this pass.

The application also decodes resources into `res/values/strings.xml`, which is how
`str_left_and_right_adapter` was resolved; resource decoding can be reproduced locally
with `jadx.cli.JadxCLI --no-src` into an ignored directory.

The restore path is implemented behind the Experimental opt-in (section 6). The
phone-side call settings, touch lock, firmware parsing and auto-off remain unimplemented
until they are traced to a specific model/firmware and reviewed.

## 6. B4S implementation

`BA36`/`AA36` (availability) and `BA37`/`AA37` (restore) are implemented behind the
reviewed `restoreDefaults` capability and the Experimental opt-in, with an explicit
confirmation dialog. Restore is treated as a transport-accepted action that waits for an
`AA37 00` result and never becomes "confirmed device state".

`BA3F`/`AA3F` (state) and `BA4A01`/`BA4A00` (set) are implemented behind the
`adaptiveLr` capability and the Experimental opt-in. The resolved resource
`str_left_and_right_adapter` = "Adaptive L/R Earbuds" supplies the user-facing label; the
setter writes `BA4A` and re-queries `BA3F` for confirmation.

The touch-lock, firmware, auto-off and phone-side call settings remain unimplemented.
