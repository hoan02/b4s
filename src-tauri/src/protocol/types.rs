//! Protocol types shared with frontend (serde JSON).

use serde::{Deserialize, Serialize};

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
    // Extended (best-effort indices — server-driven in official app)
    HifiLive = 4,
    Pop = 5,
    JazzRock = 6,
    Classical = 7,
    Acoustic = 8,
    BassReduce = 9,
    TrebleReduce = 10,
}

impl EqPreset {
    #[allow(dead_code)]
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Balanced),
            1 => Some(Self::BassBoost),
            2 => Some(Self::Voice),
            3 => Some(Self::Clear),
            4 => Some(Self::HifiLive),
            5 => Some(Self::Pop),
            6 => Some(Self::JazzRock),
            7 => Some(Self::Classical),
            8 => Some(Self::Acoustic),
            9 => Some(Self::BassReduce),
            10 => Some(Self::TrebleReduce),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Map UI id / label → preset (Baseus app-style names).
    #[allow(dead_code)]
    pub fn from_ui(s: &str) -> Result<Self, String> {
        let k = s.to_lowercase().replace([' ', '-', '_'], "");
        Ok(match k.as_str() {
            "bass" | "bassboost" | "powerfulbass" | "powerful" => Self::BassBoost,
            "voice" => Self::Voice,
            "clear" | "cleartreble" | "treble" => Self::Clear,
            "hifi" | "hifilive" | "hi-filive" => Self::HifiLive,
            "pop" | "popular" => Self::Pop,
            "jazz" | "jazzrock" | "jazzy" => Self::JazzRock,
            "classical" | "classicorchestra" => Self::Classical,
            "acoustic" => Self::Acoustic,
            "bassreduce" | "reducebass" | "lessbass" => Self::BassReduce,
            "treblereduce" | "reducetreble" | "lesstreble" => Self::TrebleReduce,
            "classic" | "baseusclassic" | "balanced" => Self::Balanced,
            _ => return Err(format!("Unknown EQ preset: {s}")),
        })
    }
}

/// Spatial / panoramic sound (app: BA43 + mode byte; BA5E for capability query).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpatialMode {
    Off = 0x00,
    Music = 0x01,
    Cinema = 0x02,
    Game = 0x03,
}

impl SpatialMode {
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

// ---------------------------------------------------------------------------
// Commands (app → device)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EqBand {
    pub frequency: u16,
    pub q_value: f32,
    pub gain: f32,
    pub filter: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Command {
    SetAnc {
        mode: AncMode,
        level: u8,
    },
    SetNoise {
        mode: AncMode,
        parameter: u8,
    },
    SetEq(EqPreset),
    SetEqIndex(u8),
    SetCustomEq {
        dict_sort: u8,
        anc: bool,
        bands: Vec<EqBand>,
    },
    QueryEq,
    /// Official app: BA02 → battery report AA02
    QueryBattery,
    /// Official app: BA27 separately requests charging-case battery (AA27).
    QueryCaseBattery,
    SetGameMode(bool),
    /// Spatial on → BA43 + mode; off → BA43 00 (common mode)
    SetSpatial(SpatialMode),
    QuerySpatial,
    /// Bass boost electronic: 0–3 (best-effort BA opcode)
    SetBassBoost(u8),
    QueryBassBoost,
    SetLdac(bool),
    SetHearingProtection {
        enabled: bool,
        level: u8,
    },
    QueryLdac,
    QueryHearingProtection,
    FindBuds(bool),
    QueryInEar,
    SetInEar(bool),
    QueryMultipoint,
    SetMultipoint(bool),
    QueryRestoreSupport,
    RestoreDefaults,
    QueryAdaptiveLr,
    SetAdaptiveLr(bool),
    QueryGesture(u8),
    SetGesture {
        layout: u8,
        left: Option<u8>,
        right: Option<u8>,
    },
}

// ---------------------------------------------------------------------------
// Events (device → app)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum DeviceEvent {
    Battery(BatteryState),
    Anc {
        mode: AncMode,
        parameter: u8,
    },
    Eq(EqPreset),
    EqIndex(u8),
    GameMode(bool),
    BassBoost(u8),
    SpatialEnabled(bool),
    Ldac(bool),
    HearingProtection {
        enabled: bool,
        level: u8,
    },
    GestureConfig {
        layout: u8,
        left: u8,
        right: u8,
    },
    InEar(bool),
    Multipoint(bool),
    RestoreAvailable(bool),
    RestoreResult(u8),
    AdaptiveLr(bool),
    /// Raw / unknown — forwarded for debug
    Unknown {
        cmd: u8,
        payload: Vec<u8>,
    },
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

#[cfg(test)]
mod intent_tests {
    use super::EqPreset;

    #[test]
    fn invalid_eq_does_not_become_balanced() {
        assert!(EqPreset::from_ui("not-a-preset").is_err());
        assert_eq!(EqPreset::from_ui("balanced").unwrap(), EqPreset::Balanced);
        assert_eq!(
            EqPreset::from_ui("powerful bass").unwrap(),
            EqPreset::BassBoost
        );
    }
}
