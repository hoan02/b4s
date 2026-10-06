//! Bass BP1 Pro ANC protocol decoder / helpers
//!
//! Packet table verified on hardware (see docs/protocol/bp1-pro-anc.md).

use crate::protocol::types::*;
use crate::protocol::Frame;

/// Decoder/command table verified from Bass BP1 Pro captures.
///
/// Other catalog models are deliberately marked as experimental. They may
/// share the AA/BA framing, but must be validated with their own captures and
/// model-specific GATT configuration before being considered supported.
pub struct Bp1ProAnc;

impl Bp1ProAnc {
    /// Decode a notification frame into a high-level event.
    pub fn decode_frame(
        frame: &Frame,
        _last_anc: Option<AncMode>,
    ) -> Result<DeviceEvent, DecodeError> {
        match frame.cmd {
            // Battery L/R: AA 02 [L%] 00 [R%] 01
            0x02 => Self::decode_battery(&frame.payload),

            // Game mode state: AA 23 [00|01]
            0x23 => {
                let on = match frame.payload.as_slice() {
                    [0] => false,
                    [1] => true,
                    _ => return Err(DecodeError::UnknownOpcode(frame.cmd)),
                };
                Ok(DeviceEvent::GameMode(on))
            }

            // Case battery: AA 27 [case%] [charging]
            0x27 => Self::decode_case(&frame.payload),

            // Do NOT map 0x32/0x33 → ANC. Those opcodes appear in other features;
            // false positives forced UI back to "Giảm ồn" when user picked Normal/Ambient.

            // ANC set/query reply: AA 34 [mode] [level?]
            // Wire modes match BA34: 00=Off, 01=ANC, 02=Transparency (docs + encode_command)
            0x34 => match Self::resolve_anc_state(&frame.payload) {
                Some((mode, parameter)) => Ok(DeviceEvent::Anc { mode, parameter }),
                None => Err(DecodeError::UnknownOpcode(0x34)),
            },

            // APK 2.17.0.1: AA30 is dictSort; AA42 is spatial-related state.
            0x30 => match frame.payload.as_slice() {
                [index] => Ok(DeviceEvent::EqIndex(*index)),
                _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
            },

            0x42 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::SpatialEnabled(false)),
                [1] => Ok(DeviceEvent::SpatialEnabled(true)),
                _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
            },
            0x53 => Self::bass_level_from_payload(&frame.payload)
                .map(DeviceEvent::BassBoost)
                .ok_or(DecodeError::UnknownOpcode(frame.cmd)),
            // In-ear switch state: AA 25 [00|01]. AA26 is only the set reply.
            0x25 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::InEar(false)),
                [1] => Ok(DeviceEvent::InEar(true)),
                _ => Err(DecodeError::UnknownOpcode(0x25)),
            },
            // Multipoint (dual-connection) state: AA 57 [00|01]. AA58 is the set reply.
            0x57 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::Multipoint(false)),
                [1] => Ok(DeviceEvent::Multipoint(true)),
                _ => Err(DecodeError::UnknownOpcode(0x57)),
            },
            // Restore-defaults availability: AA 36 [00|01] (BleCommandUtil.e == 1).
            0x36 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::RestoreAvailable(false)),
                [1] => Ok(DeviceEvent::RestoreAvailable(true)),
                _ => Err(DecodeError::UnknownOpcode(0x36)),
            },
            // Restore-defaults result: AA 37 [code]; 00 success, 0C/0D conflict.
            0x37 => match frame.payload.as_slice() {
                [code, ..] => Ok(DeviceEvent::RestoreResult(*code)),
                _ => Err(DecodeError::UnknownOpcode(0x37)),
            },
            // Gesture v1 configuration: AA 21 [layout] [left] [right].
            0x21 => match frame.payload.as_slice() {
                [layout, left, right] if *layout <= 5 => Ok(DeviceEvent::GestureConfig {
                    layout: *layout,
                    left: *left,
                    right: *right,
                }),
                _ => Err(DecodeError::UnknownOpcode(0x21)),
            },
            0x74 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::Ldac(true)),
                [1] => Ok(DeviceEvent::Ldac(false)),
                _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
            },
            // App 2.17.0.1 consumes AA93 + enabled + level as state. AA94 is
            // only a write acknowledgement and never updates confirmed state.
            0x93 => {
                if frame.payload.len() < 2 {
                    return Err(DecodeError::PayloadTooShort {
                        opcode: frame.cmd,
                        need: 2,
                        got: frame.payload.len(),
                    });
                }
                if frame.payload[0] > 1 {
                    return Err(DecodeError::UnknownOpcode(frame.cmd));
                }
                Ok(DeviceEvent::HearingProtection {
                    enabled: frame.payload[0] == 1,
                    level: frame.payload[1],
                })
            }

            // Keepalive / identity / case event — ignore or unknown
            0x12 | 0x24 | 0x43 | 0x80 => Err(DecodeError::UnknownOpcode(frame.cmd)),

            other => Err(DecodeError::UnknownOpcode(other)),
        }
    }

    /// Resolve AA 34 noise-mode state notification.
    ///
    /// Official write: `BA 34 <mode> <level>` with mode `00|01|02`.
    /// Many firmwares echo the same; some only send a 1-byte "ok" (`01`).
    /// A one-byte success ACK is not state. Preserve the returned parameter so
    /// command confirmation can match ANC level or transparency submode too.
    pub fn resolve_anc_state(payload: &[u8]) -> Option<(AncMode, u8)> {
        if payload.len() != 2 {
            return None;
        }
        let parameter = *payload.get(1)?;
        let mode = match payload[0] {
            0 => AncMode::Off,
            1 => AncMode::Anc,
            2 => AncMode::Transparency,
            _ => return None,
        };
        Some((mode, parameter))
    }

    /// Some firmwares encode charging as high bit (0x80 | pct). Strip for display.
    fn pct_and_charge(b: u8) -> (u8, bool) {
        if b > 100 {
            ((b & 0x7F).min(100), b & 0x80 != 0)
        } else {
            (b, false)
        }
    }

    /// True if byte is a plausible battery % (0–100 or 0x80|pct).
    fn is_pct_byte(b: u8) -> bool {
        b <= 100 || (b & 0x7F) <= 100
    }

    fn decode_battery(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // Exact AA02 LL 00 RR 01 layout; percentages do not determine validity.
        if payload.len() == 4
            && payload[1] == 0
            && payload[3] == 1
            && Self::is_pct_byte(payload[0])
            && Self::is_pct_byte(payload[2])
        {
            let (left, left_charging) = Self::pct_and_charge(payload[0]);
            let (right, right_charging) = Self::pct_and_charge(payload[2]);
            return Ok(DeviceEvent::Battery(BatteryState {
                left,
                right,
                case: 0,
                left_charging,
                right_charging,
                case_charging: false,
            }));
        }
        Err(DecodeError::PayloadTooShort {
            opcode: 2,
            need: 4,
            got: payload.len(),
        })
    }

    fn decode_case(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // Official: AA27 + first data byte = case % (hex)
        // HomeBleDataResolvePresenter.a0: Integer.parseInt(str.substring(4, 6), 16)
        if payload.is_empty() {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x27,
                need: 1,
                got: 0,
            });
        }
        if !Self::is_pct_byte(payload[0]) {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x27,
                need: 1,
                got: payload.len(),
            });
        }
        let (case, ch_hi) = Self::pct_and_charge(payload[0]);
        let case_charging = payload.get(1).copied().unwrap_or(0) != 0 || ch_hi;
        Ok(DeviceEvent::Battery(BatteryState {
            left: 0,
            right: 0,
            case,
            left_charging: false,
            right_charging: false,
            case_charging,
        }))
    }
}

// ---------------------------------------------------------------------------
// Command builders
// ---------------------------------------------------------------------------

impl Bp1ProAnc {
    /// Source-traced BA31 preset filters, eight bytes per filter, no invented
    /// ANC byte. Quantization truncates exactly as the Android consumer does.
    pub fn cmd_set_eq_filters(index: u8, filters: &[EqBand]) -> Vec<u8> {
        let mut payload = vec![index];
        for filter in filters {
            payload.extend_from_slice(&filter.frequency.to_le_bytes());
            payload.extend_from_slice(&((filter.gain * 10.0 + 120.0) as u16).to_le_bytes());
            payload.extend_from_slice(&((filter.q_value * 10.0) as u16).to_le_bytes());
            payload.extend_from_slice(&(filter.filter as u16).to_le_bytes());
        }
        Frame::write(0x31, &payload).encode_write()
    }

    /// BP1 Pro custom uses the same filter layout as presets. The ANC selector
    /// belongs to Storm 1 and is rejected by the model router for BP1 Pro.
    pub fn cmd_set_custom_eq(dict_sort: u8, _anc: bool, bands: &[EqBand]) -> Vec<u8> {
        Self::cmd_set_eq_filters(dict_sort, bands)
    }
    fn bass_level_from_payload(payload: &[u8]) -> Option<u8> {
        match payload {
            [enabled @ 0..=1] => Some(*enabled),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn cmd_set_anc(mode: AncMode, strength_pct: u8) -> Vec<u8> {
        let level = mode.level_from_percent(strength_pct);
        crate::protocol::encode_command(Command::SetAnc { mode, level })
    }

    pub fn cmd_set_noise(mode: AncMode, parameter: u8) -> Vec<u8> {
        crate::protocol::encode_command(Command::SetNoise { mode, parameter })
    }

    #[allow(dead_code)]
    pub fn cmd_set_eq(preset: EqPreset) -> Vec<u8> {
        crate::protocol::encode_command(Command::SetEq(preset))
    }

    pub fn cmd_set_game_mode(on: bool) -> Vec<u8> {
        crate::protocol::encode_command(Command::SetGameMode(on))
    }

    pub fn cmd_find_buds(start: bool) -> Vec<u8> {
        crate::protocol::encode_command(Command::FindBuds(start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::framing::Frame;
    use crate::protocol::{encode_command, init_state_payload};

    fn dec(raw: &[u8]) -> Result<DeviceEvent, DecodeError> {
        let f = Frame::decode_notify(raw).unwrap();
        Bp1ProAnc::decode_frame(&f, None)
    }

    #[test]
    fn battery_100_100() {
        let ev = dec(&[0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]).unwrap();
        match ev {
            DeviceEvent::Battery(b) => {
                assert_eq!(b.left, 100);
                assert_eq!(b.right, 100);
            }
            _ => panic!("expected battery"),
        }
    }

    #[test]
    fn battery_rejects_junk_prefix_and_compact_ack() {
        assert!(dec(&[0xAA, 2, 1, 5, 2, 100, 0, 80, 1]).is_err());
        assert!(dec(&[0xAA, 2, 3, 1]).is_err());
    }

    #[test]
    fn exact_low_battery_and_zero_are_values() {
        for percentage in 0..=4 {
            let DeviceEvent::Battery(battery) =
                dec(&[0xAA, 2, percentage, 0, percentage, 1]).unwrap()
            else {
                panic!("battery expected");
            };
            assert_eq!(battery.left, percentage);
            assert_eq!(battery.right, percentage);
        }
    }

    #[test]
    fn case_50_not_charging() {
        let ev = dec(&[0xAA, 0x27, 0x32, 0x00]).unwrap();
        match ev {
            DeviceEvent::Battery(b) => {
                assert_eq!(b.case, 50);
                assert!(!b.case_charging);
            }
            _ => panic!("expected case battery"),
        }
    }

    #[test]
    fn case_report_can_contain_only_the_percentage() {
        let ev = dec(&[0xAA, 0x27, 0x64]).unwrap();
        match ev {
            DeviceEvent::Battery(b) => {
                assert_eq!(b.case, 100);
                assert!(!b.case_charging);
                assert_eq!((b.left, b.right), (0, 0));
            }
            _ => panic!("expected case battery"),
        }
    }

    #[test]
    fn game_mode_on() {
        assert_eq!(
            dec(&[0xAA, 0x23, 0x01]).unwrap(),
            DeviceEvent::GameMode(true)
        );
    }

    #[test]
    fn spatial_enabled_notification_is_state_while_ack_is_not() {
        assert_eq!(
            dec(&[0xAA, 0x42, 0x01]).unwrap(),
            DeviceEvent::SpatialEnabled(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x42, 0x00]).unwrap(),
            DeviceEvent::SpatialEnabled(false)
        );
        assert!(dec(&[0xAA, 0x43, 0x01]).is_err());
    }

    #[test]
    fn empty_invalid_and_ack_only_payloads_never_create_state() {
        for packet in [
            vec![0xAA, 0x23],
            vec![0xAA, 0x23, 2],
            vec![0xAA, 0x30],
            vec![0xAA, 0x30, 0, 1],
            vec![0xAA, 0x43, 1],
            vec![0xAA, 0x74],
            vec![0xAA, 0x74, 2],
            vec![0xAA, 0x75, 1],
            vec![0xAA, 0x34, 1],
        ] {
            assert!(dec(&packet).is_err(), "unexpected state from {packet:02X?}");
        }
    }

    #[test]
    fn eq_bass() {
        assert_eq!(dec(&[0xAA, 0x30, 0x01]).unwrap(), DeviceEvent::EqIndex(1));
    }

    #[test]
    fn anc_write_bytes() {
        let bytes = Bp1ProAnc::cmd_set_anc(AncMode::Anc, 70);
        assert_eq!(bytes[0], 0xBA);
        assert_eq!(bytes[1], 0x34);
        assert_eq!(bytes[2], 0x01); // mode ANC
    }

    #[test]
    fn anc_state_retains_mode_and_parameter() {
        assert_eq!(
            Bp1ProAnc::resolve_anc_state(&[0x00, 0xFF]),
            Some((AncMode::Off, 0xFF))
        );
        assert_eq!(
            Bp1ProAnc::resolve_anc_state(&[0x02, 0x01]),
            Some((AncMode::Transparency, 1))
        );
        assert_eq!(
            Bp1ProAnc::resolve_anc_state(&[0x01, 0x05]),
            Some((AncMode::Anc, 5))
        );
    }

    #[test]
    fn anc_short_ack_and_unknown_mode_never_become_confirmed_state() {
        // AA34 01 alone is generic success, not evidence of any mode.
        assert_eq!(Bp1ProAnc::resolve_anc_state(&[0x01]), None);
        assert_eq!(Bp1ProAnc::resolve_anc_state(&[0x03, 0x02]), None);
        assert_eq!(Bp1ProAnc::resolve_anc_state(&[0x01, 0x05, 0x00]), None);
    }

    #[test]
    fn anc_no_false_aa33() {
        // Old bug: opcode 0x33 forced ANC
        assert!(dec(&[0xAA, 0x33]).is_err());
        assert!(dec(&[0xAA, 0x32]).is_err());
    }

    #[test]
    fn apk_noise_modes_use_ba34_mode_and_parameter() {
        assert_eq!(
            Bp1ProAnc::cmd_set_noise(AncMode::Off, 0xFF),
            vec![0xBA, 0x34, 0x00, 0xFF]
        );
        assert_eq!(
            Bp1ProAnc::cmd_set_noise(AncMode::Transparency, 0xFF),
            vec![0xBA, 0x34, 0x02, 0xFF]
        );
        assert_eq!(
            Bp1ProAnc::cmd_set_noise(AncMode::Anc, 5),
            vec![0xBA, 0x34, 0x01, 0x05]
        );
        assert_eq!(
            Bp1ProAnc::cmd_set_noise(AncMode::Anc, 108),
            vec![0xBA, 0x34, 0x01, 0x6C]
        );
    }

    #[test]
    fn bass_boost_uses_dedicated_ba54_command() {
        assert_eq!(
            encode_command(Command::SetBassBoost(1)),
            vec![0xBA, 0x54, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetBassBoost(0)),
            vec![0xBA, 0x54, 0x00]
        );
    }

    #[test]
    fn bass_query_is_binary_and_set_ack_is_not_state() {
        assert_eq!(dec(&[0xAA, 0x53, 1]).unwrap(), DeviceEvent::BassBoost(1));
        assert_eq!(dec(&[0xAA, 0x53, 0]).unwrap(), DeviceEvent::BassBoost(0));
        assert!(dec(&[0xAA, 0x54, 1]).is_err());
        assert!(dec(&[0xAA, 0x53, 2]).is_err());
    }

    #[test]
    fn find_buds_has_start_and_stop_commands() {
        assert_eq!(Bp1ProAnc::cmd_find_buds(true), vec![0xBA, 0x10, 0x02, 0x01]);
        assert_eq!(
            Bp1ProAnc::cmd_find_buds(false),
            vec![0xBA, 0x10, 0x02, 0x00]
        );
    }

    #[test]
    fn ldac_toggle_uses_ba75_inverted_flag_from_apk() {
        assert_eq!(
            encode_command(Command::SetLdac(true)),
            vec![0xBA, 0x75, 0x00]
        );
        assert_eq!(
            encode_command(Command::SetLdac(false)),
            vec![0xBA, 0x75, 0x01]
        );
    }

    #[test]
    fn hearing_protection_includes_enable_and_level() {
        assert_eq!(
            encode_command(Command::SetHearingProtection {
                enabled: true,
                level: 3
            }),
            vec![0xBA, 0x94, 0x01, 0x03]
        );
    }

    #[test]
    fn hearing_query_updates_state_but_short_write_ack_does_not() {
        assert_eq!(
            encode_command(Command::QueryHearingProtection),
            vec![0xBA, 0x93]
        );
        assert_eq!(
            dec(&[0xAA, 0x93, 0x01, 85]).unwrap(),
            DeviceEvent::HearingProtection {
                enabled: true,
                level: 85,
            }
        );
        assert_eq!(
            dec(&[0xAA, 0x93, 0x00, 0xFF]).unwrap(),
            DeviceEvent::HearingProtection {
                enabled: false,
                level: 0xFF,
            }
        );
        assert!(dec(&[0xAA, 0x93]).is_err());
        assert!(dec(&[0xAA, 0x94, 0x01]).is_err());
        assert!(dec(&[0xAA, 0x94, 0x01, 85]).is_err());
        assert!(dec(&[0xAA, 0x93, 0x02, 85]).is_err());
    }

    #[test]
    fn init_state_is_plain_utf8_payload() {
        assert_eq!(init_state_payload(), b"#InitState:");
    }
    #[test]
    fn source_preset_filters_use_little_endian_u16_fields() {
        let filters = [EqBand {
            frequency: 190,
            gain: -5.8,
            q_value: 0.64,
            filter: 1,
        }];
        assert_eq!(
            Bp1ProAnc::cmd_set_eq_filters(0, &filters),
            vec![0xBA, 0x31, 0, 190, 0, 62, 0, 6, 0, 1, 0]
        );
        let filters = [EqBand {
            frequency: 1000,
            gain: 0.0,
            q_value: 32.0,
            filter: 2,
        }];
        assert_eq!(
            Bp1ProAnc::cmd_set_eq_filters(10, &filters),
            vec![0xBA, 0x31, 10, 0xE8, 3, 120, 0, 0x40, 1, 2, 0]
        );
    }

    #[test]
    fn eq_readback_preserves_wire_index_and_ignores_spatial_and_ack() {
        assert_eq!(dec(&[0xAA, 0x30, 101]).unwrap(), DeviceEvent::EqIndex(101));
        assert_eq!(
            dec(&[0xAA, 0x42, 1]).unwrap(),
            DeviceEvent::SpatialEnabled(true)
        );
        assert!(dec(&[0xAA, 0x43, 1]).is_err());
        assert!(dec(&[0xAA, 0x30, 1, 2]).is_err());
    }

    #[test]
    fn bass_errors_and_invalid_layouts_never_become_level_three() {
        for payload in [
            vec![],
            vec![0x0B],
            vec![0x0C],
            vec![0x0D],
            vec![4],
            vec![2, 1],
            vec![0, 2],
            vec![1, 4],
            vec![1, 2, 0],
        ] {
            let mut frame = vec![0xAA, 0x54];
            frame.extend(payload);
            assert!(dec(&frame).is_err());
        }
    }

    #[test]
    fn in_ear_state_and_set_reply_are_distinguished() {
        assert_eq!(dec(&[0xAA, 0x25, 0x01]).unwrap(), DeviceEvent::InEar(true));
        assert_eq!(dec(&[0xAA, 0x25, 0x00]).unwrap(), DeviceEvent::InEar(false));
        // AA26 is only the set acknowledgement and must not create state.
        assert!(dec(&[0xAA, 0x26, 0x00]).is_err());
        assert!(dec(&[0xAA, 0x26, 0x01]).is_err());
        assert!(dec(&[0xAA, 0x25, 0x02]).is_err());
        assert!(dec(&[0xAA, 0x25]).is_err());
    }

    #[test]
    fn gesture_v1_state_requires_layout_and_two_side_bytes() {
        assert_eq!(
            dec(&[0xAA, 0x21, 0x03, 0x01, 0x02]).unwrap(),
            DeviceEvent::GestureConfig {
                layout: 0x03,
                left: 0x01,
                right: 0x02,
            }
        );
        // AA22 is only the set acknowledgement.
        assert!(dec(&[0xAA, 0x22, 0x03, 0x01, 0x02]).is_err());
        assert!(dec(&[0xAA, 0x21, 0x03, 0x01]).is_err());
        assert!(dec(&[0xAA, 0x21, 0x09, 0x01, 0x02]).is_err());
    }

    #[test]
    fn multipoint_state_and_set_reply_are_distinguished() {
        assert_eq!(
            dec(&[0xAA, 0x57, 0x01]).unwrap(),
            DeviceEvent::Multipoint(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x57, 0x00]).unwrap(),
            DeviceEvent::Multipoint(false)
        );
        assert!(dec(&[0xAA, 0x58, 0x01]).is_err());
        assert!(dec(&[0xAA, 0x57, 0x02]).is_err());
        assert!(dec(&[0xAA, 0x57]).is_err());
        assert_eq!(encode_command(Command::QueryMultipoint), vec![0xBA, 0x57]);
        assert_eq!(
            encode_command(Command::SetMultipoint(true)),
            vec![0xBA, 0x58, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetMultipoint(false)),
            vec![0xBA, 0x58, 0x00]
        );
    }

    #[test]
    fn restore_availability_and_result_decode() {
        assert_eq!(
            dec(&[0xAA, 0x36, 0x01]).unwrap(),
            DeviceEvent::RestoreAvailable(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x36, 0x00]).unwrap(),
            DeviceEvent::RestoreAvailable(false)
        );
        assert!(dec(&[0xAA, 0x36, 0x02]).is_err());
        assert_eq!(
            dec(&[0xAA, 0x37, 0x00]).unwrap(),
            DeviceEvent::RestoreResult(0)
        );
        assert_eq!(
            dec(&[0xAA, 0x37, 0x0C]).unwrap(),
            DeviceEvent::RestoreResult(0x0C)
        );
        assert!(dec(&[0xAA, 0x37]).is_err());
        assert_eq!(
            encode_command(Command::QueryRestoreSupport),
            vec![0xBA, 0x36]
        );
        assert_eq!(encode_command(Command::RestoreDefaults), vec![0xBA, 0x37]);
    }

    #[test]
    fn in_ear_and_gesture_command_bytes_match_the_source() {
        assert_eq!(encode_command(Command::QueryInEar), vec![0xBA, 0x25]);
        assert_eq!(
            encode_command(Command::SetInEar(true)),
            vec![0xBA, 0x26, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetInEar(false)),
            vec![0xBA, 0x26, 0x00]
        );
        assert_eq!(
            encode_command(Command::QueryGesture(0x03)),
            vec![0xBA, 0x21, 0x03]
        );
        assert_eq!(
            encode_command(Command::SetGesture {
                layout: 0x03,
                left: Some(0x01),
                right: None,
            }),
            vec![0xBA, 0x22, 0x03, 0x01, 0xFF]
        );
        assert_eq!(
            encode_command(Command::SetGesture {
                layout: 0x00,
                left: None,
                right: Some(0x03),
            }),
            vec![0xBA, 0x22, 0x00, 0xFF, 0x03]
        );
        assert_eq!(
            encode_command(Command::SetGesture {
                layout: 0x02,
                left: Some(0x04),
                right: Some(0x04),
            }),
            vec![0xBA, 0x22, 0x02, 0x04, 0x04]
        );
    }
}
