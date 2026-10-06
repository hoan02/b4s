//! Project typed model/profile metadata into protocol-facing model views.
//!
//! Control support comes from reviewed JSON profiles. Public catalog records are
//! passive scan metadata and never inherit protocol or capability defaults.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportLevel {
    Verified,
    Experimental,
    ScanOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolFamily {
    /// Packet table verified on Bass BP1 Pro / Ultra hardware.
    Bp1Pro,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub anc: bool,
    pub eq: bool,
    pub game_mode: bool,
    pub bass_boost: bool,
    pub ldac: bool,
    pub hearing_protection: bool,
    pub spatial: bool,
    pub gesture: bool,
    pub in_ear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BleTransportConfig {
    pub service_uuid: Option<String>,
    pub write_uuid: Option<String>,
    pub notify_uuid: Option<String>,
    pub use_self_uuid: bool,
    pub required_advertised_service: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub name_patterns: Vec<String>,
    pub support: SupportLevel,
    pub protocol: ProtocolFamily,
    pub has_anc: bool,
    pub has_eq: bool,
    pub has_game_mode: bool,
    pub category: String,
    /// UI grouping (from official app families)
    pub group: String,
    pub capabilities: ModelCapabilities,
    pub transport: BleTransportConfig,
    pub color_variants: Vec<String>,
    pub image_url: Option<String>,
    pub image_provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoiseCapability {
    pub supports_adaptive: bool,
    pub environments: Vec<u16>,
    pub max_custom_level: u8,
    pub supports_transparency_voice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    pub capabilities: crate::catalog::Capabilities,
    pub connection: Option<crate::catalog::ConnectionProfile>,
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub firmware: Option<String>,
    pub protocol: ProtocolFamily,
    pub verified: bool,
    pub noise: NoiseCapability,
    #[serde(default)]
    pub experimental_features: Vec<String>,
}

pub fn unknown_profile(model_id: Option<&str>, model_name: Option<&str>) -> DeviceProfile {
    DeviceProfile {
        connection: None,
        capabilities: Default::default(),
        model_id: model_id.map(str::to_owned),
        model_name: model_name.map(str::to_owned),
        firmware: None,
        protocol: ProtocolFamily::Unknown,
        verified: false,
        noise: NoiseCapability {
            supports_adaptive: false,
            environments: Vec::new(),
            max_custom_level: 0,
            supports_transparency_voice: false,
        },
        experimental_features: Vec::new(),
    }
}

fn support_level(value: &str) -> SupportLevel {
    match value {
        "verified" => SupportLevel::Verified,
        "experimental" => SupportLevel::Experimental,
        _ => SupportLevel::ScanOnly,
    }
}

fn protocol_family(value: &str) -> ProtocolFamily {
    match value {
        "bp1" => ProtocolFamily::Bp1Pro,
        _ => ProtocolFamily::Unknown,
    }
}

fn model_info_from_profile(profile: &crate::catalog::ModelProfile) -> ModelInfo {
    let capabilities = ModelCapabilities {
        anc: profile.capabilities.anc,
        eq: profile.capabilities.eq,
        game_mode: profile.capabilities.game_mode,
        bass_boost: profile.capabilities.bass_boost,
        ldac: profile.capabilities.ldac,
        hearing_protection: profile.capabilities.hearing_protection,
        spatial: profile.capabilities.spatial,
        gesture: profile.capabilities.gesture,
        in_ear: profile.capabilities.in_ear,
    };
    let connection = profile.connection.as_ref();
    let (service_uuid, write_uuid, notify_uuid, use_self_uuid, required_advertised_service) =
        match connection {
            Some(connection)
                if connection.transport == crate::catalog::ControlTransport::BleGatt =>
            {
                (
                    connection.service_uuid.clone(),
                    connection.write_uuid.clone(),
                    connection.notify_uuid.clone(),
                    false,
                    true,
                )
            }
            _ => (None, None, None, false, false),
        };
    ModelInfo {
        id: profile.id.clone(),
        display_name: profile.display_name.clone(),
        name_patterns: profile
            .aliases
            .iter()
            .map(|alias| alias.to_lowercase())
            .collect(),
        support: support_level(&profile.support),
        protocol: protocol_family(&profile.protocol_family),
        has_anc: capabilities.anc,
        has_eq: capabilities.eq,
        has_game_mode: capabilities.game_mode,
        category: profile.category.clone(),
        group: profile.group.clone(),
        capabilities,
        transport: BleTransportConfig {
            service_uuid,
            write_uuid,
            notify_uuid,
            use_self_uuid,
            required_advertised_service,
        },
        color_variants: Vec::new(),
        image_url: profile.image.clone(),
        image_provenance: "reviewed-profile".into(),
    }
}

fn scan_only_model(public: &crate::catalog::public::PublicModel) -> ModelInfo {
    ModelInfo {
        id: public.id.clone(),
        display_name: public.model.clone(),
        name_patterns: public.name_patterns(),
        support: SupportLevel::ScanOnly,
        protocol: ProtocolFamily::Unknown,
        has_anc: false,
        has_eq: false,
        has_game_mode: false,
        category: "audio".into(),
        group: public.group(),
        capabilities: ModelCapabilities {
            anc: false,
            eq: false,
            game_mode: false,
            bass_boost: false,
            ldac: false,
            hearing_protection: false,
            spatial: false,
            gesture: false,
            in_ear: false,
        },
        transport: BleTransportConfig {
            service_uuid: None,
            write_uuid: None,
            notify_uuid: None,
            use_self_uuid: false,
            required_advertised_service: false,
        },
        color_variants: public.color_codes(),
        image_url: None,
        image_provenance: "offline-public-metadata".into(),
    }
}

pub fn profile_for(
    model_id: Option<&str>,
    model_name: Option<&str>,
    firmware: Option<&str>,
) -> DeviceProfile {
    let Some(id) = model_id else {
        return unknown_profile(None, model_name);
    };
    if let Some(profile) = crate::catalog::profile_for(id) {
        if profile.support == "scanOnly" {
            let mut passive = unknown_profile(Some(&profile.id), Some(&profile.display_name));
            passive.firmware = firmware.map(str::to_owned);
            return passive;
        }
        return DeviceProfile {
            connection: profile.connection,
            capabilities: profile.capabilities,
            model_id: Some(profile.id),
            model_name: Some(profile.display_name),
            firmware: firmware.map(str::to_owned),
            protocol: protocol_family(&profile.protocol_family),
            verified: profile.support == "verified",
            noise: NoiseCapability {
                supports_adaptive: profile.noise.supports_adaptive,
                environments: profile.noise.environments,
                max_custom_level: profile.noise.max_custom_level,
                supports_transparency_voice: profile.noise.supports_transparency_voice,
            },
            experimental_features: profile.experimental_features,
        };
    }
    let Some(model) = all_models().into_iter().find(|model| model.id == id) else {
        return unknown_profile(Some(id), model_name);
    };
    if model.support != SupportLevel::ScanOnly {
        return unknown_profile(Some(id), Some(&model.display_name));
    }
    DeviceProfile {
        connection: None,
        capabilities: crate::catalog::Capabilities::default(),
        model_id: Some(model.id),
        model_name: Some(model.display_name),
        firmware: firmware.map(str::to_owned),
        protocol: ProtocolFamily::Unknown,
        verified: false,
        noise: NoiseCapability {
            supports_adaptive: false,
            environments: Vec::new(),
            max_custom_level: 0,
            supports_transparency_voice: false,
        },
        experimental_features: Vec::new(),
    }
}

/// Runtime identity data comes only from reviewed profiles and the public scan catalog.
pub fn all_models() -> Vec<ModelInfo> {
    let mut models: Vec<_> = crate::catalog::all_profiles()
        .iter()
        .map(model_info_from_profile)
        .collect();
    merge_public_models(&mut models);
    models
}

fn merge_public_models(models: &mut Vec<ModelInfo>) {
    use crate::catalog::public::{headphone_models, identity_key};

    for public in headphone_models() {
        if let Some(existing) = models
            .iter_mut()
            .find(|model| identity_key(&model.display_name) == identity_key(&public.model))
        {
            for pattern in public.name_patterns() {
                if !existing
                    .name_patterns
                    .iter()
                    .any(|alias| identity_key(alias) == identity_key(&pattern))
                {
                    existing.name_patterns.push(pattern);
                }
            }
            existing.color_variants = public.color_codes();
        } else {
            models.push(scan_only_model(public));
        }
    }
}

/// Resolve a complete advertised product name or an explicit reviewed alias.
pub fn identify(ble_name: &str) -> Option<ModelInfo> {
    let identity = crate::catalog::public::identity_key(ble_name);
    all_models().into_iter().find(|model| {
        crate::catalog::public::identity_key(&model.display_name) == identity
            || model
                .name_patterns
                .iter()
                .any(|alias| crate::catalog::public::identity_key(alias) == identity)
    })
}

pub fn catalog_json() -> Vec<ModelInfo> {
    all_models()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_profile_is_the_only_control_source() {
        let model = identify("Baseus Bass BP1 Pro").unwrap();
        assert_eq!(model.id, "bass-bp1-pro");
        assert_eq!(model.support, SupportLevel::Verified);
        assert_eq!(model.protocol, ProtocolFamily::Bp1Pro);

        let profile = profile_for(Some(&model.id), None, Some("1.0.0"));
        assert!(profile.verified);
        assert_eq!(profile.noise.max_custom_level, 5);
        assert_eq!(profile.noise.environments, vec![101, 102, 103, 108]);
        assert!(profile.connection.is_some());
    }

    #[test]
    fn public_metadata_is_scan_only_and_never_supplies_capabilities_or_transport() {
        let model = identify("Baseus Bowie MA10").unwrap();
        assert_eq!(model.support, SupportLevel::ScanOnly);
        assert_eq!(model.protocol, ProtocolFamily::Unknown);
        assert!(!model.capabilities.eq);
        assert!(model.transport.write_uuid.is_none());

        let profile = profile_for(Some(&model.id), None, None);
        assert!(!profile.verified);
        assert_eq!(profile.protocol, ProtocolFamily::Unknown);
        assert_eq!(profile.connection, None);
        assert!(!profile.capabilities.eq);
        assert!(profile.noise.environments.is_empty());
        assert!(crate::protocol::encode_feature(
            &profile,
            crate::protocol::FeatureCommand::FindBuds(true)
        )
        .is_err());
    }

    #[test]
    fn model_matching_requires_a_complete_exact_identity_or_alias() {
        assert!(identify("random Baseus Bass BP1 Pro clone").is_none());
        let model = identify("BP1 Pro").unwrap();
        assert_eq!(model.id, "bass-bp1-pro");
    }

    #[test]
    fn headphone_catalog_is_discoverable_but_speakers_are_not_headphones() {
        for public in crate::catalog::public::headphone_models() {
            let resolved = identify(&public.model).unwrap();
            assert_eq!(
                crate::catalog::public::identity_key(&resolved.display_name),
                crate::catalog::public::identity_key(&public.model)
            );
        }
        assert!(identify("Baseus Sleep SK1").is_none());
    }

    #[test]
    fn bp1_ultra_remains_explicitly_unresolved() {
        let model = identify("Baseus Bass BP1 Ultra").unwrap();
        assert_eq!(model.id, "bass-bp1-ultra");
        assert_eq!(model.support, SupportLevel::ScanOnly);
        assert_eq!(model.protocol, ProtocolFamily::Unknown);
        assert!(model.transport.write_uuid.is_none());
        let profile = profile_for(Some(&model.id), None, None);
        assert!(profile.connection.is_none());
        assert!(!profile.capabilities.anc);
        assert!(!profile.capabilities.eq);
        assert!(!profile.verified);
    }

    #[test]
    fn unknown_profile_has_no_control_contract() {
        let profile = profile_for(Some("unknown"), Some("Unknown Earbuds"), None);
        assert!(!profile.verified);
        assert_eq!(profile.protocol, ProtocolFamily::Unknown);
        assert!(profile.connection.is_none());
        assert!(profile.noise.environments.is_empty());
    }
}
