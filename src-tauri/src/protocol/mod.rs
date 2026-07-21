//! Baseus earbuds protocol
//!
//! Reverse-engineered from official Baseus Android app + live BLE captures
//! on Bass BP1 Pro ANC (source: elaxptr/baseus-desktop, MIT).
//!
//! Frame format:
//!   Notify (device → app):  AA <cmd> <payload...>
//!   Write  (app → device):  BA <cmd> <payload...>
//!
//! GATT (BP1 Pro ANC):
//!   Service : 53527aa4-29f7-ae11-4e74-997334782568
//!   Write   : ee684b1a-1e9b-ed3e-ee55-f894667e92ac
//!   Notify  : 654b749c-e37f-ae1f-ebab-40ca133e3690

mod framing;
mod types;
mod bp1_pro;
pub mod models;

pub use framing::{Frame, FrameError};
pub use types::*;
pub use bp1_pro::Bp1ProAnc;
pub use models::{
    catalog_json, identify as identify_model, looks_like_baseus, ModelInfo,
    ProtocolFamily, SupportLevel,
};

/// Encode a command for writing to the GATT write characteristic.
pub fn encode_command(cmd: Command) -> Vec<u8> {
    match cmd {
        Command::SetAnc { mode, level } => {
            Frame::write(0x34, &[mode.to_byte(), level]).encode_write()
        }
        Command::SetEq(preset) => {
            Frame::write(0x43, &[preset.to_byte()]).encode_write()
        }
        Command::QueryEq => {
            Frame::write(0x42, &[]).encode_write()
        }
        Command::SetGameMode(on) => {
            Frame::write(0x24, &[if on { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::FindBuds => {
            // Gesture / find opcode candidate — may need model-specific tweak
            Frame::write(0x92, &[0x01]).encode_write()
        }
    }
}

/// Decode a notification payload from the device.
pub fn decode_notification(
    data: &[u8],
    last_anc: Option<AncMode>,
) -> Result<DeviceEvent, DecodeError> {
    let frame = Frame::decode_notify(data)?;
    Bp1ProAnc::decode_frame(&frame, last_anc)
}
