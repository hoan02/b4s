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

pub mod advertisement;
mod crc_table;
mod families;
mod framing;
pub mod models;
pub mod receiver;
pub mod router;
mod types;
pub mod wrap_v2;

pub use families::bp1::Bp1ProAnc;
pub use framing::Frame;
pub use models::{
    catalog_json, identify as identify_model, profile_for, DeviceProfile, ModelInfo,
    ProtocolFamily, SupportLevel,
};
pub use router::{
    decode_frame, encode_feature, encode_listening, FeatureCommand, ListeningCommand,
};
pub use types::*;
pub use wrap_v2::{unwrap_notify, wrap_ba_command};

/// Encode a bare BA command (no 789C wrap).
pub fn encode_command(cmd: Command) -> Vec<u8> {
    match cmd {
        Command::SetAnc { mode, level } => {
            Frame::write(0x34, &[mode.to_byte(), level]).encode_write()
        }
        Command::SetNoise { mode, parameter } => {
            Frame::write(0x34, &[mode.to_byte(), parameter]).encode_write()
        }
        Command::SetEq(preset) => Frame::write(0x43, &[preset.to_byte()]).encode_write(),
        Command::SetEqIndex(index) => Frame::write(0x43, &[index]).encode_write(),
        Command::SetCustomEq {
            dict_sort,
            anc,
            bands,
        } => Bp1ProAnc::cmd_set_custom_eq(dict_sort, anc, &bands),
        Command::QueryAnc => Frame::write(0x33, &[]).encode_write(),
        Command::QueryGameMode => Frame::write(0x23, &[]).encode_write(),
        Command::QueryEq => Frame::write(0x30, &[]).encode_write(),
        Command::QueryBattery => {
            // EarphoneFunctionShowFragmentNewUI: companion.c(model, "BA02", sn)
            Frame::write(0x02, &[]).encode_write()
        }
        Command::QueryCaseBattery => Frame::write(0x27, &[]).encode_write(),
        Command::SetGameMode(on) => {
            Frame::write(0x24, &[if on { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::QuerySpatial => Frame::write(0x42, &[]).encode_write(),
        Command::SetSpatial(mode) => {
            // PanoramicSoundViewModel.u: "BA43" + "00"|"01"|"02"|…
            Frame::write(0x43, &[mode.to_byte()]).encode_write()
        }
        Command::QueryBassBoost => Frame::write(0x53, &[]).encode_write(),
        Command::SetBassBoost(level) => Frame::write(0x54, &[level]).encode_write(),
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
        Command::QueryInEar => Frame::write(0x25, &[]).encode_write(),
        Command::SetInEar(enabled) => {
            // GestureBleManager.a + EarHeadSetViewModel.o0: BA26 01 on / 00 off.
            Frame::write(0x26, &[if enabled { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::QueryMultipoint => Frame::write(0x57, &[]).encode_write(),
        Command::SetMultipoint(enabled) => {
            // EarphoneFunctionShowFragmentNewUI.Setting.n: BA58 01 on / 00 off.
            Frame::write(0x58, &[if enabled { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::QueryRestoreSupport => Frame::write(0x36, &[]).encode_write(),
        Command::RestoreDefaults => {
            // EarPhoneSettingV2Activity.u2: BA37 after the user confirms; AA37 is the result.
            Frame::write(0x37, &[]).encode_write()
        }
        Command::QueryAdaptiveLr => Frame::write(0x3F, &[]).encode_write(),
        Command::SetAdaptiveLr(enabled) => {
            // EarPhoneSettingV2Activity.e2: BA4A01 on / BA4A00 off.
            Frame::write(0x4A, &[if enabled { 0x01 } else { 0x00 }]).encode_write()
        }
        Command::QueryGesture(layout) => Frame::write(0x21, &[layout]).encode_write(),
        Command::SetGesture {
            layout,
            left,
            right,
        } => {
            // GestureSettingViewModel.Z: BA22 <layout> <left|FF> <right|FF>.
            Frame::write(0x22, &[layout, left.unwrap_or(0xFF), right.unwrap_or(0xFF)])
                .encode_write()
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
    framing: crate::catalog::WireFraming,
) -> Result<DeviceEvent, DecodeError> {
    let frames = match framing {
        crate::catalog::WireFraming::BareAaBa if data.first() == Some(&0xAA) => vec![data.to_vec()],
        crate::catalog::WireFraming::Headphone789c if data.starts_with(&[0x78, 0x9C]) => {
            unwrap_notify(data)
        }
        crate::catalog::WireFraming::Headphone789c => Vec::new(),
        crate::catalog::WireFraming::BareAaBa | crate::catalog::WireFraming::Unresolved => {
            Vec::new()
        }
    };
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
