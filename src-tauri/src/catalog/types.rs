use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlTransport { BleGatt, Unresolved }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WireFraming { BareAaBa, Headphone789c, Unresolved }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionProfile {
    pub transport: ControlTransport,
    pub framing: WireFraming,
    pub service_uuid: Option<String>,
    pub write_uuid: Option<String>,
    pub notify_uuid: Option<String>,
    pub handshake: Vec<u8>,
    pub init_state_query: bool,
    pub firmware_versions: Vec<String>,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default)]
    pub anc: bool,
    #[serde(default)]
    pub eq: bool,
    #[serde(default)]
    pub custom_eq: bool,
    #[serde(default)]
    pub game_mode: bool,
    #[serde(default)]
    pub bass_boost: bool,
    #[serde(default)]
    pub spatial: bool,
    #[serde(default)]
    pub ldac: bool,
    #[serde(default)]
    pub hearing_protection: bool,
    #[serde(default)]
    pub find_buds: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EqPresetProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub dict_sort: u8,
    pub curve: Vec<f32>,
    #[serde(default)]
    pub filters: Vec<crate::protocol::EqBand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EqProfile {
    pub bands: Vec<u16>,
    pub min_gain: f32,
    pub max_gain: f32,
    pub custom_slots: u8,
    pub presets: Vec<EqPresetProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NoiseProfile {
    pub supports_adaptive: bool,
    pub environments: Vec<u16>,
    pub max_custom_level: u8,
    pub supports_transparency_voice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    #[serde(default = "legacy_schema")]
    pub schema_version: u8,
    #[serde(default)]
    pub connection: Option<ConnectionProfile>,
    pub id: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub support: String,
    pub protocol_family: String,
    pub category: String,
    pub group: String,
    pub capabilities: Capabilities,
    pub noise: NoiseProfile,
    pub eq: Option<EqProfile>,
    pub image: Option<String>,
}

fn legacy_schema() -> u8 { 1 }
