//! Source/hardware evidence describes knowledge; it never authorizes commands.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceStatus {
    Unknown,
    Unsupported,
    SourceReviewed,
    Implemented,
    HardwareVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeatureEvidence {
    pub status: EvidenceStatus,
    pub provenance: String,
    /// Empty means scope is unknown, not that every firmware was tested.
    pub firmware_versions: Vec<String>,
}

pub fn unknown_features() -> std::collections::BTreeMap<String, FeatureEvidence> {
    FEATURE_KEYS
        .iter()
        .map(|key| {
            (
                (*key).to_owned(),
                FeatureEvidence {
                    status: EvidenceStatus::Unknown,
                    provenance: String::new(),
                    firmware_versions: Vec::new(),
                },
            )
        })
        .collect()
}

pub const FEATURE_KEYS: &[&str] = &[
    "anc",
    "eq",
    "customEq",
    "gameMode",
    "bassBoost",
    "spatial",
    "ldac",
    "hearingProtection",
    "findBuds",
    "gesture",
    "inEar",
    "multipoint",
    "restoreDefaults",
    "adaptiveLr",
    "windNoise",
    "callEnhancement",
    "batteryEnhancement",
    "soundBalance",
    "deviceManagement",
    "gestureV2",
    "personalizedSound",
    "firmwareUpdate",
    "aiServices",
];
