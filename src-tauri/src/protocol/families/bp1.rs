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

/// How a family reports the transparency sub-mode in AA33.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseLayout {
    /// `HeadPhoneMainActivity`/`NoiseReduceManger`: one transparency mode.
    SingleTransparency,
    /// `EarPodNewActivity`/`NoiseReduceManger2`: type1 selects full or voice.
    SelectableTransparency,
}

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

            // Noise state: AA 33 [mode] [type] [type1] [type2]
            // (NoiseReduceManger.u). The BP1 Pro family has a single
            // transparency mode, so the retained type1 selector is not state.
            0x33 => Self::decode_noise_state(&frame.payload, NoiseLayout::SingleTransparency),

            // AA 34 [status] is only the BA34 result (NoiseReduceManger.s):
            // 01 success (the app then re-queries BA33), 00/other = rejected.
            0x34 | 0x52 => Self::decode_write_status(frame),

            // APK 2.17.0.1: AA30 is dictSort.
            0x30 => match frame.payload.as_slice() {
                [index] => Ok(DeviceEvent::EqIndex(*index)),
                _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
            },

            // PanoramicSoundViewModel.x: selected mode, optionally followed by
            // the retained mode. 00 off, 01 music, 02 cinema, 03 game.
            0x42 => match frame.payload.as_slice() {
                [mode @ 0..=3] | [mode @ 0..=3, 0..=3] => {
                    Ok(DeviceEvent::SpatialMode(Self::spatial_mode(*mode)))
                }
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
            // Multipoint (dual-connection) state: AA 57 [..] [00|01]. The app
            // reads the trailing byte (endsWith "01"). AA58 is the set reply.
            0x57 => Self::trailing_flag(&frame.payload)
                .map(DeviceEvent::Multipoint)
                .ok_or(DecodeError::UnknownOpcode(0x57)),
            // Restore-defaults availability: AA 36 [00|01] (BleCommandUtil.e == 1).
            0x36 => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::RestoreAvailable(false)),
                [1] => Ok(DeviceEvent::RestoreAvailable(true)),
                _ => Err(DecodeError::UnknownOpcode(0x36)),
            },
            // Restore-defaults result: AA 37 [code]. EarPhoneSettingActivity and
            // EarPhoneSettingV2Activity treat 01 as success; 0C dual connection
            // and 0D in-call are rejections.
            0x37 => match frame.payload.as_slice() {
                [code, ..] => Ok(DeviceEvent::RestoreResult(*code)),
                _ => Err(DecodeError::UnknownOpcode(0x37)),
            },
            // Adaptive L/R earbuds state: AA 3F [00|01]. AA4A is the set reply.
            0x3F => match frame.payload.as_slice() {
                [0] => Ok(DeviceEvent::AdaptiveLr(false)),
                [1] => Ok(DeviceEvent::AdaptiveLr(true)),
                _ => Err(DecodeError::UnknownOpcode(0x3F)),
            },
            // Wind-noise reduction state: AA 51 [..] [00|01] (ResultHandle.h/j
            // reads the trailing byte). AA52 is only the set status.
            0x51 => Self::trailing_flag(&frame.payload)
                .map(DeviceEvent::WindNoise)
                .ok_or(DecodeError::UnknownOpcode(0x51)),
            // Gesture contract selector pushed by firmware (GestureSettingViewModel,
            // HeadPhoneMainActivity, EarPodNewActivity): AA 8B 01 means V2.
            0x8B => match frame.payload.as_slice() {
                [flag @ 0..=1, ..] => Ok(DeviceEvent::GestureProtocolV2(*flag == 1)),
                _ => Err(DecodeError::UnknownOpcode(0x8B)),
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
            // LdacSettingActivity: switch.setChecked(AA74 byte == "01").
            0x74 => match frame.payload.as_slice() {
                [1] => Ok(DeviceEvent::Ldac(true)),
                [0] => Ok(DeviceEvent::Ldac(false)),
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

    /// Decode the AA33 noise report shared by both families.
    ///
    /// `mode, type, type1, type2`: the firmware retains the inactive selectors,
    /// so the parameter is normalized for the active mode. Shorter replies are
    /// accepted as the app does (`NoiseReduceManger.u` defaults missing bytes).
    /// Canonical B4S transparency parameters are FF (full) and 01 (voice).
    pub fn decode_noise_state(
        payload: &[u8],
        layout: NoiseLayout,
    ) -> Result<DeviceEvent, DecodeError> {
        let (mode, parameter) = match (payload, layout) {
            ([0, _, ..], _) => (AncMode::Off, 0xFF),
            ([1, parameter, ..], _) => (AncMode::Anc, *parameter),
            ([2, _, ..], NoiseLayout::SingleTransparency) => (AncMode::Transparency, 0xFF),
            // Ultra's selector is 01 full / 02 voice on the wire.
            ([2, _, 1, ..], NoiseLayout::SelectableTransparency) => (AncMode::Transparency, 0xFF),
            ([2, _, 2, ..], NoiseLayout::SelectableTransparency) => (AncMode::Transparency, 1),
            _ => return Err(DecodeError::UnknownOpcode(0x33)),
        };
        Ok(DeviceEvent::Anc { mode, parameter })
    }

    /// One-byte set status (AA34, AA52). Longer payloads are not this reply.
    fn decode_write_status(frame: &Frame) -> Result<DeviceEvent, DecodeError> {
        match frame.payload.as_slice() {
            [code] => Ok(DeviceEvent::WriteStatus {
                opcode: frame.cmd,
                code: *code,
            }),
            _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
        }
    }

    /// Switch state carried in the trailing byte (`endsWith("01")` in the app).
    fn trailing_flag(payload: &[u8]) -> Option<bool> {
        match payload.last() {
            Some(0) => Some(false),
            Some(1) => Some(true),
            _ => None,
        }
    }

    pub fn spatial_mode(mode: u8) -> SpatialMode {
        match mode {
            0 => SpatialMode::Off,
            1 => SpatialMode::Music,
            2 => SpatialMode::Cinema,
            _ => SpatialMode::Game,
        }
    }

    /// Unverified: no APK consumer reads a charging bit (`BleUtils.d` and the
    /// case handler take the raw value). Kept from the original BP1 Pro notes
    /// until a capture settles it; values above 100 never show above 100%.
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
        // EarBatteryManger.g clamps the raw case value to 0..100.
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

    pub fn cmd_find_buds(side: FindSide, start: bool) -> Vec<u8> {
        crate::protocol::encode_command(Command::FindBuds { side, start })
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
    fn case_values_above_100_are_clamped_like_the_app() {
        let DeviceEvent::Battery(battery) = dec(&[0xAA, 0x27, 0x6E]).unwrap() else {
            panic!("case battery expected");
        };
        assert_eq!(battery.case, 100);
        assert!(!battery.case_charging);
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
    fn spatial_report_is_the_selected_mode_while_ack_is_not() {
        assert_eq!(
            dec(&[0xAA, 0x42, 0x01]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Music)
        );
        assert_eq!(
            dec(&[0xAA, 0x42, 0x00]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Off)
        );
        // Cinema and the two-byte selected/retained form are state too.
        assert_eq!(
            dec(&[0xAA, 0x42, 0x02]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Cinema)
        );
        assert_eq!(
            dec(&[0xAA, 0x42, 0x00, 0x02]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Off)
        );
        assert!(dec(&[0xAA, 0x42, 0x04]).is_err());
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
            vec![0xAA, 0x34],
            vec![0xAA, 0x34, 1, 5],
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
    fn aa33_reports_noise_state_for_the_active_mode() {
        for (wire, mode, parameter) in [
            (vec![0xAA, 0x33, 0, 104, 1, 1], AncMode::Off, 0xFF),
            (vec![0xAA, 0x33, 1, 104, 1, 1], AncMode::Anc, 104),
            (vec![0xAA, 0x33, 1, 3], AncMode::Anc, 3),
            // Single-transparency family: the retained type1 is not a submode.
            (vec![0xAA, 0x33, 2, 104, 2, 1], AncMode::Transparency, 0xFF),
            (vec![0xAA, 0x33, 2, 0xFF], AncMode::Transparency, 0xFF),
        ] {
            assert_eq!(dec(&wire).unwrap(), DeviceEvent::Anc { mode, parameter });
        }
        assert!(dec(&[0xAA, 0x33]).is_err());
        assert!(dec(&[0xAA, 0x33, 1]).is_err());
        assert!(dec(&[0xAA, 0x33, 3, 1]).is_err());
        assert!(dec(&[0xAA, 0x32]).is_err());
    }

    #[test]
    fn aa34_is_only_a_write_status_never_noise_state() {
        assert_eq!(
            dec(&[0xAA, 0x34, 0x01]).unwrap(),
            DeviceEvent::WriteStatus {
                opcode: 0x34,
                code: 0x01
            }
        );
        assert_eq!(
            dec(&[0xAA, 0x34, 0x00]).unwrap(),
            DeviceEvent::WriteStatus {
                opcode: 0x34,
                code: 0x00
            }
        );
        // The old two-byte "state" layout is not an app-consumed reply.
        assert!(dec(&[0xAA, 0x34, 0x02, 0xFF]).is_err());
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
    fn find_buds_has_per_side_start_and_stop_commands() {
        assert_eq!(
            Bp1ProAnc::cmd_find_buds(FindSide::Both, true),
            vec![0xBA, 0x10, 0x02, 0x01]
        );
        assert_eq!(
            Bp1ProAnc::cmd_find_buds(FindSide::Both, false),
            vec![0xBA, 0x10, 0x02, 0x00]
        );
        assert_eq!(
            Bp1ProAnc::cmd_find_buds(FindSide::Left, true),
            vec![0xBA, 0x10, 0x00, 0x01]
        );
        assert_eq!(
            Bp1ProAnc::cmd_find_buds(FindSide::Right, false),
            vec![0xBA, 0x10, 0x01, 0x00]
        );
    }

    #[test]
    fn ldac_uses_01_for_enabled_on_write_and_readback() {
        assert_eq!(
            encode_command(Command::SetLdac(true)),
            vec![0xBA, 0x75, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetLdac(false)),
            vec![0xBA, 0x75, 0x00]
        );
        assert_eq!(dec(&[0xAA, 0x74, 0x01]).unwrap(), DeviceEvent::Ldac(true));
        assert_eq!(dec(&[0xAA, 0x74, 0x00]).unwrap(), DeviceEvent::Ldac(false));
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
            DeviceEvent::SpatialMode(SpatialMode::Music)
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
        // The app reads the trailing byte.
        assert_eq!(
            dec(&[0xAA, 0x57, 0x05, 0x01]).unwrap(),
            DeviceEvent::Multipoint(true)
        );
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
            dec(&[0xAA, 0x37, 0x01]).unwrap(),
            DeviceEvent::RestoreResult(1)
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
    fn wind_noise_state_and_set_reply_are_distinguished() {
        assert_eq!(
            dec(&[0xAA, 0x51, 0x01]).unwrap(),
            DeviceEvent::WindNoise(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x51, 0x00]).unwrap(),
            DeviceEvent::WindNoise(false)
        );
        assert!(dec(&[0xAA, 0x51, 0x02]).is_err());
        assert!(dec(&[0xAA, 0x51]).is_err());
        assert_eq!(
            dec(&[0xAA, 0x51, 0x00, 0x01]).unwrap(),
            DeviceEvent::WindNoise(true)
        );
        // AA52 is the set status or an error code (0A/0B/0C), never state.
        assert_eq!(
            dec(&[0xAA, 0x52, 0x0A]).unwrap(),
            DeviceEvent::WriteStatus {
                opcode: 0x52,
                code: 0x0A
            }
        );
        assert_eq!(encode_command(Command::QueryWindNoise), vec![0xBA, 0x51]);
        assert_eq!(
            encode_command(Command::SetWindNoise(true)),
            vec![0xBA, 0x52, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetWindNoise(false)),
            vec![0xBA, 0x52, 0x00]
        );
    }

    #[test]
    fn adaptive_lr_state_and_set_code() {
        assert_eq!(
            dec(&[0xAA, 0x3F, 0x01]).unwrap(),
            DeviceEvent::AdaptiveLr(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x3F, 0x00]).unwrap(),
            DeviceEvent::AdaptiveLr(false)
        );
        assert!(dec(&[0xAA, 0x3F, 0x02]).is_err());
        assert!(dec(&[0xAA, 0x4A, 0x01]).is_err());
        assert_eq!(encode_command(Command::QueryAdaptiveLr), vec![0xBA, 0x3F]);
        assert_eq!(
            encode_command(Command::SetAdaptiveLr(true)),
            vec![0xBA, 0x4A, 0x01]
        );
        assert_eq!(
            encode_command(Command::SetAdaptiveLr(false)),
            vec![0xBA, 0x4A, 0x00]
        );
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

    #[test]
    fn firmware_gesture_contract_selector_is_decoded() {
        assert_eq!(
            dec(&[0xAA, 0x8B, 0x01]).unwrap(),
            DeviceEvent::GestureProtocolV2(true)
        );
        assert_eq!(
            dec(&[0xAA, 0x8B, 0x00]).unwrap(),
            DeviceEvent::GestureProtocolV2(false)
        );
        assert!(dec(&[0xAA, 0x8B]).is_err());
        assert!(dec(&[0xAA, 0x8B, 0x02]).is_err());
    }
}
