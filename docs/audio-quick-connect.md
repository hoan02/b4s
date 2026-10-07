# Windows audio quick connect

The pairing screen offers **Quick connect** when Windows can associate the current default audio output with a supported BLE control device. The card refreshes when the app regains focus and every 15 seconds while pairing is idle. It uses the default playback role, not the communications role or an individual application's audio routing override.

The backend obtains the output using `MediaDevice.GetDefaultAudioRenderId`, reads its `System.Devices.ContainerId`, and enumerates OS-known Bluetooth LE interfaces with `BluetoothLEDevice.GetDeviceSelector`. Only matching, non-empty container identities are accepted. Device names resolve the reviewed model profile after identity matching; they never establish that two endpoints belong to the same earbuds.

Windows-known addresses are opened through btleplug's `add_peripheral`, so this path starts no BLE advertisement scan. Both advertised identities and Windows identities use the same catalog resolver. The existing connection code still discovers and validates the exact control service, characteristics and handshake before sending model commands. The adapter event listener is installed even when no scan has run, preserving disconnect handling.

The selected audio endpoint is revalidated before preparing the connection. Unsupported profiles and multiple supported BLE candidates are rejected. Successful connections use the existing remembered-device store. Audio associations are re-read from Windows each time rather than trusting a stale name or persisted MAC mapping.

When automatic reconnect is enabled and there is a remembered device, it first tries the current audio output through this same direct path. If Windows has no supported audio association, it retains the previous bounded scan fallback. An identified audio target that fails preparation does not silently fall back to a different previously used pair.

If Windows has not enumerated the LE endpoint, uses a different container for the audio and control radios, or the LE name cannot resolve a reviewed profile, use **Scan for devices** and select the control entry manually. Random/offset MAC addresses are never guessed. macOS and Linux retain their existing pairing flow.

## Validation on hardware

1. Select supported earbuds as the Windows default playback output, then focus B4S. Check that the quick-connect card identifies that output.
2. Connect from the card without scanning; check battery/state notifications and disconnect handling.
3. Switch output to the built-in speakers or another pair. Verify that a stale card does not initiate a connection to the previous pair.
4. Test two pairs with the same name. Only matching container identities may be selected; ambiguous control endpoints require manual selection.
5. Test Bluetooth off, an unenumerated LE endpoint, and a model with separate audio/LE identities. Manual scanning remains available.

References: [default audio output API](https://learn.microsoft.com/en-us/uwp/api/windows.media.devices.mediadevice.getdefaultaudiorenderid), [device identity properties](https://learn.microsoft.com/en-us/windows/uwp/devices-sensors/device-information-properties), [audio endpoint container IDs](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/audio-endpoint-container-id).
