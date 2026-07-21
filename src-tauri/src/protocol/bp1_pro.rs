//! Bass BP1 Pro ANC protocol decoder / helpers
//!
//! Packet table verified on hardware (see docs/protocol/bp1-pro-anc.md).

use super::framing::Frame;
use super::types::*;

pub struct Bp1ProAnc;

impl Bp1ProAnc {
    /// Decode a notification frame into a high-level event.
    pub fn decode_frame(
        frame: &Frame,
        last_anc: Option<AncMode>,
    ) -> Result<DeviceEvent, DecodeError> {
        match frame.cmd {
            // Battery L/R: AA 02 [L%] 00 [R%] 01
            0x02 => Self::decode_battery(&frame.payload),

            // Game mode state: AA 23 [00|01]
            0x23 => {
                let on = frame.payload.first().copied().unwrap_or(0) != 0;
                Ok(DeviceEvent::GameMode(on))
            }

            // Case battery: AA 27 [case%] [charging]
            0x27 => Self::decode_case(&frame.payload),

            // ANC direct notifies (some firmwares)
            0x32 => Ok(DeviceEvent::Anc(AncMode::Transparency)),
            0x33 => Ok(DeviceEvent::Anc(AncMode::Anc)),

            // ANC ack: AA 34 [payload] — firmware-dependent
            0x34 => {
                let mode = Self::resolve_anc_ack(&frame.payload, last_anc);
                Ok(DeviceEvent::Anc(mode))
            }

            // EQ query response / set ack
            0x42 | 0x43 => {
                let byte = frame.payload.first().copied().unwrap_or(0);
                let preset = EqPreset::from_byte(byte).unwrap_or(EqPreset::Balanced);
                Ok(DeviceEvent::Eq(preset))
            }

            // Keepalive / identity / case event — ignore or unknown
            0x12 | 0x24 | 0x30 | 0x80 => Err(DecodeError::UnknownOpcode(frame.cmd)),

            other => Err(DecodeError::UnknownOpcode(other)),
        }
    }

    /// Resolve AA 34 ANC ack.
    /// Zero payload → Off; non-zero → last commanded mode (firmware variance).
    pub fn resolve_anc_ack(payload: &[u8], last_commanded: Option<AncMode>) -> AncMode {
        if payload.first().copied().unwrap_or(0) == 0 {
            AncMode::Off
        } else {
            // If last command was Off but firmware sends flat 0x01 ack, keep Off
            match last_commanded {
                Some(AncMode::Off) => AncMode::Off,
                Some(m) => m,
                None => AncMode::Anc,
            }
        }
    }

    fn decode_battery(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // AA 02 [left_pct] 0x00 [right_pct] 0x01
        // Bytes 1 and 3 are bud-ID markers, NOT charging flags.
        if payload.len() < 4 {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x02,
                need: 4,
                got: payload.len(),
            });
        }
        Ok(DeviceEvent::Battery(BatteryState {
            left: payload[0],
            right: payload[2],
            case: 0, // filled by 0x27
            left_charging: false,
            right_charging: false,
            case_charging: false,
        }))
    }

    fn decode_case(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // AA 27 [case_pct] [case_charging]
        if payload.len() < 2 {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x27,
                need: 2,
                got: payload.len(),
            });
        }
        Ok(DeviceEvent::Battery(BatteryState {
            left: 0,
            right: 0,
            case: payload[0],
            left_charging: false,
            right_charging: false,
            case_charging: payload[1] != 0,
        }))
    }
}

// ---------------------------------------------------------------------------
// Command builders
// ---------------------------------------------------------------------------

impl Bp1ProAnc {
    pub fn cmd_set_anc(mode: AncMode, strength_pct: u8) -> Vec<u8> {
        let level = mode.level_from_percent(strength_pct);
        super::encode_command(Command::SetAnc { mode, level })
    }

    pub fn cmd_set_eq(preset: EqPreset) -> Vec<u8> {
        super::encode_command(Command::SetEq(preset))
    }

    pub fn cmd_query_eq() -> Vec<u8> {
        super::encode_command(Command::QueryEq)
    }

    pub fn cmd_set_game_mode(on: bool) -> Vec<u8> {
        super::encode_command(Command::SetGameMode(on))
    }

    pub fn cmd_find_buds() -> Vec<u8> {
        super::encode_command(Command::FindBuds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::framing::Frame;

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
    fn game_mode_on() {
        assert_eq!(dec(&[0xAA, 0x23, 0x01]).unwrap(), DeviceEvent::GameMode(true));
    }

    #[test]
    fn eq_bass() {
        assert_eq!(
            dec(&[0xAA, 0x43, 0x01]).unwrap(),
            DeviceEvent::Eq(EqPreset::BassBoost)
        );
    }

    #[test]
    fn anc_write_bytes() {
        let bytes = Bp1ProAnc::cmd_set_anc(AncMode::Anc, 70);
        assert_eq!(bytes[0], 0xBA);
        assert_eq!(bytes[1], 0x34);
        assert_eq!(bytes[2], 0x01); // mode ANC
    }
}
