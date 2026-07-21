use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EqPresetProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub dict_sort: u8,
    pub curve: Vec<f32>,
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
    pub id: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub support: String,
    pub protocol_family: String,
    pub category: String,
    pub group: String,
    pub capabilities: serde_json::Value,
    pub noise: NoiseProfile,
    pub eq: Option<EqProfile>,
    pub image: Option<String>,
}
