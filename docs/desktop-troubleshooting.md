# Desktop troubleshooting and diagnostics

This guide covers the current desktop BLE flow. B4S controls earbuds through
their Bluetooth LE control service; a device connected as a Windows audio
endpoint is not necessarily controllable by B4S.

## Scanning and connecting

1. Confirm Bluetooth is enabled in the operating system and the adapter is
   available to other BLE applications.
2. Put the earbuds in their discoverable state, then use **Scan** in B4S.
3. If the earbuds appear but controls are unavailable, check their support level
   in the app and [model catalog](model-catalog.md). Recognition alone does not
   enable a control connection. BP1 Ultra is currently scan-only while its
   control transport remains unverified.
4. If a saved device is supported, enable **Reconnect automatically** in
   Settings to perform one scan of up to 12 seconds at app startup. This option
   is off by default. Use manual scan if the device is not found.

Restart Bluetooth or the app after changing the adapter state. If the earbuds
are connected to another phone, disconnect them there and place them back into
their discoverable state before scanning again.

## Startup and the system tray

**Start B4S at sign-in** and **Reconnect automatically** are separate settings;
both are off by default. Closing the main window hides the app to the system
tray when tray setup is available. Choose **Quit B4S** in the tray menu to exit
and let the app make bounded best-effort cleanup of scanning, find-earbuds, and
the BLE connection.

## Reporting a problem

B4S does not currently provide a diagnostic bundle or upload diagnostic data.
When reporting an issue, include only the details needed to reproduce it:

- B4S version, operating system version, and Bluetooth adapter model.
- Earbud model as shown in B4S, firmware version if known, and the support level
  shown by the app.
- The steps taken, the control used, and the visible error or unexpected state.
- Whether scanning, manual connection, or opt-in reconnect was used.

Do not attach account data, personal device identifiers, private signing keys,
firmware files, official APKs, or copyrighted decompiled source. See the
[capture guide](captures/README.md) before collecting Bluetooth traces; captures
may contain device identifiers and must be reviewed and redacted locally.

Find-earbuds can play a loud sound. Remove the earbuds before using that control.
