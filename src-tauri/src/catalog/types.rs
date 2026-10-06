use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlTransport {
    BleGatt,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WireFraming {
    BareAaBa,
    Headphone789c,
    Unresolved,
}

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
    #[serde(default)]
    pub gesture: bool,
    #[serde(default)]
    pub in_ear: bool,
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
pub struct HearingProfile {
    pub thresholds: Vec<u8>,
    pub preserve_threshold_sentinel: bool,
    pub provenance: String,
}

/// One gesture click layout and the function IDs the reviewed model accepts.
/// `layout` is the wire layout byte (0 double, 1 triple, 2 long, 3 single,
/// 4 single-press, 5 penta); `functions` are the allowed wire function IDs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GestureLayoutProfile {
    pub layout: u8,
    pub functions: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GestureProfile {
    /// True when the model exposes both left and right button mappings.
    pub dual_button: bool,
    pub layouts: Vec<GestureLayoutProfile>,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InEarProfile {
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
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
    #[serde(default)]
    pub hearing: Option<HearingProfile>,
    #[serde(default)]
    pub gesture: Option<GestureProfile>,
    #[serde(default)]
    pub in_ear: Option<InEarProfile>,
    pub image: Option<String>,
}
