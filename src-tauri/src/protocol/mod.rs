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
pub mod advertisement;
mod types;
mod bp1_pro;
mod crc_table;
pub mod wrap_v2;
pub mod models;
pub mod router;

pub use framing::Frame;
pub use types::*;
pub use bp1_pro::Bp1ProAnc;
pub use models::{
    catalog_json, identify as identify_model, looks_like_baseus, profile_for, DeviceProfile,
    ModelInfo, ProtocolFamily, SupportLevel,
};
pub use wrap_v2::{battery_query_frame, needs_v2_wrap, unwrap_notify, wrap_ba_command};
pub use router::{encode_feature, encode_listening, FeatureCommand, ListeningCommand};

/// Encode a bare BA command (no 789C wrap).
pub fn encode_command(cmd: Command) -> Vec<u8> {
    match cmd {
        Command::SetAnc { mode, level } => {
            Frame::write(0x34, &[mode.to_byte(), level]).encode_write()
        }
        Command::SetNoise { mode, parameter } => {
            Frame::write(0x34, &[mode.to_byte(), parameter]).encode_write()
        }
        Command::SetEq(preset) => {
            Frame::write(0x43, &[preset.to_byte()]).encode_write()
        }
        Command::SetEqIndex(index) => Frame::write(0x43, &[index]).encode_write(),
        Command::SetCustomEq { dict_sort, anc, bands } => {
            Bp1ProAnc::cmd_set_custom_eq(dict_sort, anc, &bands)
        }
        Command::QueryEq => {
            Frame::write(0x42, &[]).encode_write()
        }
        Command::QueryBattery => {
            // EarphoneFunctionShowFragmentNewUI: companion.c(model, "BA02", sn)
            Frame::write(0x02, &[]).encode_write()
        }
        Command::SetGameMode(on) => {
            Frame::write(0x24, &[if on { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::SetSpatial(mode) => {
            // PanoramicSoundViewModel.u: "BA43" + "00"|"01"|"02"|…
            Frame::write(0x43, &[mode.to_byte()]).encode_write()
        }
        Command::SetBassBoost(level) => {
            let level = level.min(3);
            Frame::write(0x54, &[if level > 0 { 0x01 } else { 0x00 }, level]).encode_write()
        }
        Command::SetLdac(enabled) => {
            // LdacSettingActivity.K0: BA75 + 00 when enabled, 01 when disabled.
            Frame::write(0x75, &[if enabled { 0x00 } else { 0x01 }]).encode_write()
        }
        Command::SetHearingProtection { enabled, level } => {
            Frame::write(0x94, &[if enabled { 0x01 } else { 0x00 }, level]).encode_write()
        }
        Command::QueryLdac => Frame::write(0x74, &[]).encode_write(),
        Command::QueryHearingProtection => Frame::write(0x93, &[]).encode_write(),
        Command::FindBuds(start) => {
            // Official app 2.14.1: BA100201 starts both buds; the same command
            // with the final flag cleared stops the alert.
            Frame::write(0x10, &[0x02, if start { 0x01 } else { 0x00 }]).encode_write()
        }
    }
}

#[allow(dead_code)]
pub fn init_state_payload() -> Vec<u8> {
    b"#InitState:".to_vec()
}

/// Decode a notification payload from the device (bare AA or 789C-wrapped).
/// Returns the first successfully decoded event (prefer handle_notification
/// which applies *all* frames when AA02+AA27 arrive together).
#[allow(dead_code)]
pub fn decode_notification(
    data: &[u8],
    last_anc: Option<AncMode>,
) -> Result<DeviceEvent, DecodeError> {
    let frames = unwrap_notify(data);
    let mut last_err: Option<DecodeError> = None;
    for f in frames {
        match Frame::decode_notify(&f) {
            Ok(fr) => match Bp1ProAnc::decode_frame(&fr, last_anc) {
                Ok(ev) => return Ok(ev),
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(DecodeError::Frame(e)),
        }
    }
    Err(last_err.unwrap_or_else(|| DecodeError::UnknownOpcode(data.get(1).copied().unwrap_or(0))))
}
