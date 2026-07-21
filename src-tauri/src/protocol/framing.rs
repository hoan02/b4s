//! Frame encode / decode
//!
//! Notify magic = 0xAA
//! Write  magic = 0xBA
//! No length field, no CRC on BLE path.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum FrameError {
    #[error("buffer too short: need at least {need} bytes, got {got}")]
    TooShort { need: usize, got: usize },
    #[error("bad magic: expected 0xAA or 0xBA, got {got:#04x}")]
    BadMagic { got: u8 },
}

/// Raw frame: [magic, cmd, payload...]
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub magic: u8,
    pub cmd: u8,
    pub payload: Vec<u8>,
}

impl Frame {
    pub const MAGIC_NOTIFY: u8 = 0xAA;
    pub const MAGIC_WRITE: u8 = 0xBA;

    /// Parse a GATT notification (device → app).
    pub fn decode_notify(buf: &[u8]) -> Result<Self, FrameError> {
        Self::decode_with_magic(buf, Self::MAGIC_NOTIFY)
    }

    fn decode_with_magic(buf: &[u8], expected: u8) -> Result<Self, FrameError> {
        if buf.len() < 2 {
            return Err(FrameError::TooShort {
                need: 2,
                got: buf.len(),
            });
        }
        if buf[0] != expected {
            // Some firmwares may echo AA on both paths — be lenient for notify path
            if buf[0] != Self::MAGIC_NOTIFY && buf[0] != Self::MAGIC_WRITE {
                return Err(FrameError::BadMagic { got: buf[0] });
            }
        }
        Ok(Self {
            magic: buf[0],
            cmd: buf[1],
            payload: buf[2..].to_vec(),
        })
    }

    /// Build a write frame (app → device).
    pub fn write(cmd: u8, payload: &[u8]) -> Self {
        Self {
            magic: Self::MAGIC_WRITE,
            cmd,
            payload: payload.to_vec(),
        }
    }

    /// Encode for GATT write characteristic.
    pub fn encode_write(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + self.payload.len());
        out.push(Self::MAGIC_WRITE);
        out.push(self.cmd);
        out.extend_from_slice(&self.payload);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_battery() {
        let f = Frame::decode_notify(&[0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]).unwrap();
        assert_eq!(f.cmd, 0x02);
        assert_eq!(f.payload, vec![0x64, 0x00, 0x64, 0x01]);
    }

    #[test]
    fn encode_anc_write() {
        let f = Frame::write(0x34, &[0x01, 0x68]);
        assert_eq!(f.encode_write(), vec![0xBA, 0x34, 0x01, 0x68]);
    }

    #[test]
    fn reject_bad_magic() {
        assert!(matches!(
            Frame::decode_notify(&[0x00, 0x30]),
            Err(FrameError::BadMagic { got: 0x00 })
        ));
    }
}
