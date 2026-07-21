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

            // Do NOT map 0x32/0x33 → ANC. Those opcodes appear in other features;
            // false positives forced UI back to "Giảm ồn" when user picked Normal/Ambient.

            // ANC set/query reply: AA 34 [mode] [level?]
            // Wire modes match BA34: 00=Off, 01=ANC, 02=Transparency (docs + encode_command)
            0x34 => match Self::resolve_anc_ack(&frame.payload, last_anc) {
                Some(mode) => Ok(DeviceEvent::Anc(mode)),
                None => Err(DecodeError::UnknownOpcode(0x34)),
            },

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

    /// Resolve AA 34 noise-mode notify.
    ///
    /// Official write: `BA 34 <mode> <level>` with mode `00|01|02`.
    /// Many firmwares echo the same; some only send a 1-byte "ok" (`01`).
    /// Never default to ANC when unsure — that made Normal/Ambient snap back to Giảm ồn.
    pub fn resolve_anc_ack(payload: &[u8], last_commanded: Option<AncMode>) -> Option<AncMode> {
        let b0 = payload.first().copied()?;
        match b0 {
            // Explicit mode byte (matches AncMode::to_byte)
            0x00 => Some(AncMode::Off),
            0x02 => Some(AncMode::Transparency),
            0x01 => {
                // AA34 01 <level>: real ANC report (level often 0x10..=0xFF)
                // AA34 01 alone: generic success — keep last user command
                if payload.len() >= 2 {
                    let lvl = payload[1];
                    if lvl >= 0x10 || lvl == 0x00 {
                        return Some(AncMode::Anc);
                    }
                }
                last_commanded
            }
            // Strength-only or unknown — trust last commanded mode only
            _ => last_commanded,
        }
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
        // Official BleUtils.d (app 2.14.1) — STRICT only:
        //   regex AA02([0-9A-F]{2})00([0-9A-F]{2})01
        // Full notify: AA 02 LL 00 RR 01  → payload = [LL, 00, RR, 01]
        //
        // Loose formats (compact 2-byte, junk salvage) caused % to jump when
        // mode/EQ ACKs were misread as battery. Do not re-introduce them here.

        if payload.len() >= 4 && payload[1] == 0x00 && payload[3] == 0x01 {
            if Self::is_pct_byte(payload[0]) && Self::is_pct_byte(payload[2]) {
                let (left, lc) = Self::pct_and_charge(payload[0]);
                let (right, rc) = Self::pct_and_charge(payload[2]);
                // Mode ACKs often forge tiny % (1–3). Real buds report ≥5 once linked.
                // Require both sides present and non-trivial (official always sends L+R).
                if left >= 5 && right >= 5 {
                    return Ok(DeviceEvent::Battery(BatteryState {
                        left,
                        right,
                        case: 0,
                        left_charging: lc,
                        right_charging: rc,
                        case_charging: false,
                    }));
                }
            }
        }

        // Salvage only the exact LL 00 RR 01 sub-pattern with both sides > 0
        if let Some(ev) = Self::scan_lr_pattern(payload) {
            return Ok(ev);
        }

        Err(DecodeError::PayloadTooShort {
            opcode: 0x02,
            need: 4,
            got: payload.len(),
        })
    }

    /// Find exact LL 00 RR 01 with both L,R in 1..=100 (or charge bit).
    fn scan_lr_pattern(payload: &[u8]) -> Option<DeviceEvent> {
        if payload.len() < 4 {
            return None;
        }
        for i in 0..payload.len().saturating_sub(3) {
            if payload[i + 1] == 0x00 && payload.get(i + 3) == Some(&0x01) {
                let l = payload[i];
                let r = payload[i + 2];
                if !Self::is_pct_byte(l) || !Self::is_pct_byte(r) {
                    continue;
                }
                let (left, lc) = Self::pct_and_charge(l);
                let (right, rc) = Self::pct_and_charge(r);
                // Require both sides present and non-trivial — avoids mode-ACK noise
                // like 01 00 00 01 or 03 00 01 01 from BA34/BA43 echoes
                if left >= 5 && right >= 5 {
                    return Some(DeviceEvent::Battery(BatteryState {
                        left,
                        right,
                        case: 0,
                        left_charging: lc,
                        right_charging: rc,
                        case_charging: false,
                    }));
                }
            }
        }
        None
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
        // Ignore case=0 unless charging flag — avoids wiping case on junk AA27
        if case == 0 && !ch_hi && payload.get(1).copied().unwrap_or(0) == 0 {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x27,
                need: 1,
                got: payload.len(),
            });
        }
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
    pub fn cmd_set_anc(mode: AncMode, strength_pct: u8) -> Vec<u8> {
        let level = mode.level_from_percent(strength_pct);
        super::encode_command(Command::SetAnc { mode, level })
    }

    pub fn cmd_set_eq(preset: EqPreset) -> Vec<u8> {
        super::encode_command(Command::SetEq(preset))
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
    fn battery_salvage_wrapped_junk_prefix() {
        // 789C leftover still has LL 00 RR 01 with both ≥ 5
        let ev = dec(&[0xAA, 0x02, 0x01, 0x05, 0x02, 0x64, 0x00, 0x50, 0x01]).unwrap();
        match ev {
            DeviceEvent::Battery(b) => {
                assert_eq!(b.left, 100);
                assert_eq!(b.right, 80);
            }
            _ => panic!("expected battery"),
        }
    }

    #[test]
    fn battery_reject_mode_ack_noise() {
        // Typical junk: AA02 01 00 00 01 (looks like pattern, both sides tiny)
        assert!(dec(&[0xAA, 0x02, 0x01, 0x00, 0x00, 0x01]).is_err());
        // Compact 2-byte no longer accepted (was main flicker source)
        assert!(dec(&[0xAA, 0x02, 0x03, 0x01]).is_err());
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

    #[test]
    fn anc_ack_explicit_modes() {
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x00, 0xFF], None),
            Some(AncMode::Off)
        );
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x02, 0xFF], None),
            Some(AncMode::Transparency)
        );
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x01, 0x68], None),
            Some(AncMode::Anc)
        );
    }

    #[test]
    fn anc_ack_generic_ok_keeps_last_not_default_anc() {
        // AA34 01 alone = generic ok — must not force ANC when user chose Off
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x01], Some(AncMode::Off)),
            Some(AncMode::Off)
        );
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x01], Some(AncMode::Transparency)),
            Some(AncMode::Transparency)
        );
        // No last command + vague ack → no event (was defaulting to Anc)
        assert_eq!(Bp1ProAnc::resolve_anc_ack(&[0x01], None), None);
    }

    #[test]
    fn anc_no_false_aa33() {
        // Old bug: opcode 0x33 forced ANC
        assert!(dec(&[0xAA, 0x33]).is_err());
        assert!(dec(&[0xAA, 0x32]).is_err());
    }
}
