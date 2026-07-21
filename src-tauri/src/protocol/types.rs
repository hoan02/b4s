//! Protocol types shared with frontend (serde JSON).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GATT UUIDs — BP1 Pro ANC (confirmed via nRF Connect)
// ---------------------------------------------------------------------------

pub mod uuids {
    use uuid::Uuid;

    pub fn service() -> Uuid {
        Uuid::parse_str("53527aa4-29f7-ae11-4e74-997334782568").unwrap()
    }
    pub fn write() -> Uuid {
        Uuid::parse_str("ee684b1a-1e9b-ed3e-ee55-f894667e92ac").unwrap()
    }
    pub fn notify() -> Uuid {
        Uuid::parse_str("654b749c-e37f-ae1f-ebab-40ca133e3690").unwrap()
    }

    pub const SERVICE_STR: &str = "53527aa4-29f7-ae11-4e74-997334782568";
    pub const WRITE_STR: &str = "ee684b1a-1e9b-ed3e-ee55-f894667e92ac";
    pub const NOTIFY_STR: &str = "654b749c-e37f-ae1f-ebab-40ca133e3690";
}

// ---------------------------------------------------------------------------
// Battery / Case
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    pub left: u8,
    pub right: u8,
    pub case: u8,
    pub left_charging: bool,
    pub right_charging: bool,
    pub case_charging: bool,
}

// ---------------------------------------------------------------------------
// ANC
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AncMode {
    Off,
    Anc,
    Transparency,
}

impl AncMode {
    pub fn to_byte(self) -> u8 {
        match self {
            AncMode::Off => 0x00,
            AncMode::Anc => 0x01,
            AncMode::Transparency => 0x02,
        }
    }

    pub fn default_level(self) -> u8 {
        match self {
            AncMode::Off => 0xFF,
            AncMode::Anc => 0x68, // default strength
            AncMode::Transparency => 0xFF,
        }
    }

    /// Map strength 0–100 → device level 0x10–0xFF (ANC only).
    pub fn level_from_percent(self, pct: u8) -> u8 {
        match self {
            AncMode::Anc => {
                let pct = pct.min(100) as u16;
                // Map 0..100 → 0x10..0xFF
                (0x10 + (pct * (0xFF - 0x10) / 100)) as u8
            }
            _ => self.default_level(),
        }
    }
}

// ---------------------------------------------------------------------------
// EQ
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EqPreset {
    Balanced = 0,
    BassBoost = 1,
    Voice = 2,
    Clear = 3,
}

impl EqPreset {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Balanced),
            1 => Some(Self::BassBoost),
            2 => Some(Self::Voice),
            3 => Some(Self::Clear),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }

    pub fn from_ui(s: &str) -> Self {
        match s {
            "bass" | "bassBoost" | "BassBoost" => Self::BassBoost,
            "voice" | "Voice" => Self::Voice,
            "clear" | "Clear" => Self::Clear,
            _ => Self::Balanced,
        }
    }
}

// ---------------------------------------------------------------------------
// Commands (app → device)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Command {
    SetAnc { mode: AncMode, level: u8 },
    SetEq(EqPreset),
    QueryEq,
    SetGameMode(bool),
    FindBuds,
}

// ---------------------------------------------------------------------------
// Events (device → app)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum DeviceEvent {
    Battery(BatteryState),
    Anc(AncMode),
    Eq(EqPreset),
    GameMode(bool),
    /// Raw / unknown — forwarded for debug
    Unknown { cmd: u8, payload: Vec<u8> },
}

// ---------------------------------------------------------------------------
// Decode errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    Frame(#[from] super::framing::FrameError),
    #[error("unknown opcode 0x{0:02X}")]
    UnknownOpcode(u8),
    #[error("payload too short for opcode 0x{opcode:02X}: need {need}, got {got}")]
    PayloadTooShort { opcode: u8, need: usize, got: usize },
}
