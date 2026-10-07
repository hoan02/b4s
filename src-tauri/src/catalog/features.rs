//! Canonical control feature identities shared by catalog and authorization.
#[derive(Debug, Clone, Copy)]
pub enum Feature {
    Listening,
    Eq,
    CustomEq,
    Game,
    Bass,
    Spatial,
    Ldac,
    Hearing,
    Find,
    Gesture,
    InEar,
    Multipoint,
    RestoreDefaults,
    AdaptiveLr,
}

impl Feature {
    /// Capability key used by the reviewed profile's `experimentalFeatures`.
    pub fn key(self) -> &'static str {
        match self {
            Feature::Gesture => "gesture",
            Feature::InEar => "inEar",
            Feature::Multipoint => "multipoint",
            Feature::RestoreDefaults => "restoreDefaults",
            Feature::AdaptiveLr => "adaptiveLr",
            Feature::Listening => "anc",
            Feature::Eq => "eq",
            Feature::CustomEq => "customEq",
            Feature::Game => "gameMode",
            Feature::Bass => "bassBoost",
            Feature::Spatial => "spatial",
            Feature::Ldac => "ldac",
            Feature::Hearing => "hearingProtection",
            Feature::Find => "findBuds",
        }
    }
}
