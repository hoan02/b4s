//! BP1 Ultra state layouts from APK 2.17.0.1 and Windows hardware readbacks.
use crate::protocol::{AncMode, Bp1ProAnc, DecodeError, DeviceEvent, Frame, SpatialMode};

pub struct Bp1Ultra;

impl Bp1Ultra {
    pub fn decode_frame(frame: &Frame) -> Result<DeviceEvent, DecodeError> {
        match (frame.cmd, frame.payload.as_slice()) {
            // NoiseReduceManger.u: mode, ANC type, transparency type, custom level.
            // The inactive selectors are retained by firmware. Normalize the
            // parameter for the active mode, never infer state from AA34 ACK.
            (0x33, [mode @ 0..=2, anc, _transparency, _custom]) => {
                let (mode, parameter) = match mode {
                    0 => (AncMode::Off, 0xFF),
                    1 => (AncMode::Anc, *anc),
                    _ => (AncMode::Transparency, 0xFF),
                };
                Ok(DeviceEvent::Anc { mode, parameter })
            }
            // PanoramicSoundViewModel.x/u: selected mode followed by model type.
            (0x42, [mode @ 0..=3, _retained @ 0..=3]) => Ok(DeviceEvent::SpatialMode(match mode {
                0 => SpatialMode::Off,
                1 => SpatialMode::Music,
                2 => SpatialMode::Cinema,
                _ => SpatialMode::Game,
            })),
            (0x53, [0, _level]) => Ok(DeviceEvent::BassBoost(0)),
            (0x53, [1, level @ 1..=5]) => Ok(DeviceEvent::BassBoost(*level)),
            // Only the shared layouts observed for this model are delegated.
            (0x02 | 0x27 | 0x23 | 0x30 | 0x74 | 0x93 | 0x21, _) => {
                Bp1ProAnc::decode_frame(frame, None)
            }
            _ => Err(DecodeError::UnknownOpcode(frame.cmd)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(bytes: &[u8]) -> Result<DeviceEvent, DecodeError> {
        Bp1Ultra::decode_frame(&Frame::decode_notify(bytes).unwrap())
    }

    #[test]
    fn captured_noise_selectors_follow_the_active_mode() {
        for (wire, mode, parameter) in [
            ([0xAA, 0x33, 1, 102, 1, 5], AncMode::Anc, 102),
            ([0xAA, 0x33, 0, 102, 1, 5], AncMode::Off, 255),
            ([0xAA, 0x33, 2, 102, 1, 5], AncMode::Transparency, 255),
            ([0xAA, 0x33, 1, 1, 1, 1], AncMode::Anc, 1),
        ] {
            assert_eq!(decode(&wire).unwrap(), DeviceEvent::Anc { mode, parameter });
        }
        assert!(decode(&[0xAA, 0x34, 1]).is_err());
        assert!(decode(&[0xAA, 0x34, 1, 102]).is_err());
        assert!(decode(&[0xAA, 0x33, 1]).is_err());
    }

    #[test]
    fn captured_sound_states_and_acks_are_distinct() {
        assert_eq!(
            decode(&[0xAA, 0x42, 0, 1]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Off)
        );
        assert_eq!(
            decode(&[0xAA, 0x42, 1, 1]).unwrap(),
            DeviceEvent::SpatialMode(SpatialMode::Music)
        );
        assert_eq!(
            decode(&[0xAA, 0x53, 1, 3]).unwrap(),
            DeviceEvent::BassBoost(3)
        );
        assert!(decode(&[0xAA, 0x53, 1, 6]).is_err());
        for opcode in [0x24, 0x43, 0x54, 0x75, 0x94, 0x22] {
            assert!(decode(&[0xAA, opcode, 1]).is_err());
        }
    }

    #[test]
    fn crc_valid_wrapped_captures_route_only_to_ultra_layouts() {
        use crate::protocol::{decode_frame, unwrap_notify, ProtocolFamily};
        let captures = [
            (
                vec![0x78, 0x9C, 0, 14, 2, 1, 5, 0x33, 1, 102, 1, 5, 0xA8, 0xFF],
                DeviceEvent::Anc {
                    mode: AncMode::Anc,
                    parameter: 102,
                },
            ),
            (
                vec![0x78, 0x9C, 0, 12, 2, 1, 3, 0x42, 2, 2, 0x13, 0xDA],
                DeviceEvent::SpatialMode(SpatialMode::Cinema),
            ),
            (
                vec![0x78, 0x9C, 0, 12, 2, 1, 3, 0x53, 1, 3, 0x26, 0x4B],
                DeviceEvent::BassBoost(3),
            ),
        ];
        for (wire, expected) in captures {
            let frames = unwrap_notify(&wire);
            assert_eq!(frames.len(), 1);
            let frame = Frame::decode_notify(&frames[0]).unwrap();
            assert_eq!(
                decode_frame(ProtocolFamily::Bp1Ultra, &frame, None).unwrap(),
                expected
            );
            assert!(decode_frame(ProtocolFamily::Bp1Pro, &frame, None).is_err());
        }
    }
}
