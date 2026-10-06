# BP1 Pro protocol reference

This reference applies only to the reviewed BP1 Pro profile. BP1 Ultra remains
scan-only until its model/firmware/transport path is captured. Other Baseus
models require their own evidence and profile.

Sources: reviewed BLE packet notes and official app analysis (APK 2.14.1 and
2.17.0.1). Firmware scope is not captured; see the
[implementation tracker](../headphone-desktop-progress.md).

## Reviewed BP1 Pro GATT service

| Role | UUID |
|------|------|
| Service | `53527aa4-29f7-ae11-4e74-997334782568` |
| Write | `ee684b1a-1e9b-ed3e-ee55-f894667e92ac` |
| Notify | `654b749c-e37f-ae1f-ebab-40ca133e3690` |

## Frame format

```text
Notify (device → app):  AA <cmd> <payload...>
Write  (app → device):  BA <cmd> <payload...>
```

The BP1 Pro profile declares bare AA/BA framing. Wrapped `789C` frames found
in other model paths do not apply unless a reviewed profile explicitly declares
that framing.

## Source packet notes

| Action | Bytes | Evidence boundary |
|--------|-------|-------------------|
| Handshake | `BA 05 00` | Exact profile handshake; no alternate value is attempted |
| Battery query | `BA 02` | Reviewed startup/query command |
| ANC Off | `BA 34 00 FF` | Source packet form; device state comes from notification |
| ANC On | `BA 34 01 <level>` | Model constraints are profile-scoped |
| Transparency | `BA 34 02 <mode>` | Mode must be explicit and supported |
| EQ / spatial | `BA 43 <value>` | Shared opcode; state decoder and capability decide interpretation |
| EQ query | `BA 42` | Query/readback path |
| Case battery query | `BA 27` | Query is implemented; physical hardware confirmation remains open |
| Game ON/OFF | `BA 24 01` / `BA 24 00` | Must be confirmed by state query |
| Game query | `BA 23` | Query/readback path |
| Find both buds | `BA 10 02 01` | Explicit start/stop action |
| Hearing protection state | `BA 93` | Requires a reviewed model threshold schema; not enabled for BP1 Pro |
| Hearing protection set | `BA 94 <enabled> <level>` | ACK is not device state |

## Notification notes

| Event | Bytes | Interpretation |
|-------|-------|----------------|
| Battery L/R | `AA 02 <L%> 00 <R%> 01` | Nullable readings; zero is valid |
| Case battery | `AA 27 <case%> <charging>` | First payload byte is percentage |
| ANC | `AA 34 <mode> <parameter>` | B4S accepts the exact two-byte state payload and confirms a command only when both mode and parameter match; one-byte ACK and unknown layouts do not update state |
| EQ query/state | `AA 42 …` / `AA 43 …` | A write ACK alone does not confirm requested state |
| Game state | `AA 23 <00|01>` | State query observation |
| Hearing protection | `AA 93 <enabled> <level>` | The only accepted confirmed-state opcode; `AA 94` is ACK/error only |

## Implementation boundary

B4S uses only UUIDs and framing declared by the selected profile. It does not
retry another handshake value, probe alternate characteristics, or switch to a
wrapped decoder after a bare-frame error. Unknown, malformed, ACK-only or
out-of-session observations do not update confirmed feature state.

The packet notes and offline tests do not replace hardware acceptance for every
feature/firmware. Capture identity and per-feature verification remain explicit
gates in the [tracker](../headphone-desktop-progress.md).
