pub mod public;
mod types;

pub use types::Capabilities;
pub use types::ModelProfile;
pub use types::{ConnectionProfile, ControlTransport, WireFraming};

include!(concat!(env!("OUT_DIR"), "/model_profiles.rs"));

static PROFILES: once_cell::sync::Lazy<Vec<ModelProfile>> = once_cell::sync::Lazy::new(|| {
    let profiles: Vec<ModelProfile> = PROFILE_SOURCES
        .iter()
        .map(|source| serde_json::from_str(source).expect("valid model profile JSON"))
        .collect();
    validate_profiles(&profiles).expect("valid model catalog");
    profiles
});

pub fn all_profiles() -> Vec<ModelProfile> {
    PROFILES.clone()
}

pub fn profile_for(model_id: &str) -> Option<ModelProfile> {
    all_profiles()
        .into_iter()
        .find(|profile| profile.id == model_id)
}

#[allow(dead_code)]
pub fn validate() -> Result<(), String> {
    validate_profiles(&all_profiles())
}

fn validate_profiles(profiles: &[ModelProfile]) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for profile in profiles {
        if profile.capabilities.hearing_protection && profile.hearing.is_none() {
            return Err(format!("missing hearing constraints in {}", profile.id));
        }
        if let Some(hearing) = &profile.hearing {
            let unique: std::collections::HashSet<_> = hearing.thresholds.iter().collect();
            if hearing.provenance.trim().is_empty()
                || hearing.thresholds.is_empty()
                || unique.len() != hearing.thresholds.len()
                || hearing
                    .thresholds
                    .iter()
                    .any(|value| ![75, 80, 85, 90, 95, 100].contains(value))
            {
                return Err(format!("invalid hearing constraints in {}", profile.id));
            }
        }
        if profile.capabilities.gesture && profile.gesture.is_none() {
            return Err(format!("missing gesture constraints in {}", profile.id));
        }
        if let Some(gesture) = &profile.gesture {
            if gesture.provenance.trim().is_empty() || gesture.layouts.is_empty() {
                return Err(format!("invalid gesture constraints in {}", profile.id));
            }
            let mut layouts = std::collections::HashSet::new();
            for layout in &gesture.layouts {
                if layout.layout > 5 || !layouts.insert(layout.layout) {
                    return Err(format!("invalid gesture layout in {}", profile.id));
                }
                if layout.functions.is_empty()
                    || layout
                        .functions
                        .iter()
                        .any(|function| !matches!(function, 0..=19 | 27 | 28))
                {
                    return Err(format!("invalid gesture functions in {}", profile.id));
                }
            }
        }
        if profile.capabilities.in_ear && profile.in_ear.is_none() {
            return Err(format!("missing in-ear provenance in {}", profile.id));
        }
        if let Some(in_ear) = &profile.in_ear {
            if in_ear.provenance.trim().is_empty() {
                return Err(format!("invalid in-ear provenance in {}", profile.id));
            }
        }
        if profile.capabilities.multipoint && profile.multipoint.is_none() {
            return Err(format!("missing multipoint provenance in {}", profile.id));
        }
        if let Some(multipoint) = &profile.multipoint {
            if multipoint.provenance.trim().is_empty() {
                return Err(format!("invalid multipoint provenance in {}", profile.id));
            }
        }
        let mut experimental = std::collections::HashSet::new();
        for feature in &profile.experimental_features {
            let capability_enabled = match feature.as_str() {
                "gesture" => profile.capabilities.gesture && profile.gesture.is_some(),
                "inEar" => profile.capabilities.in_ear && profile.in_ear.is_some(),
                "multipoint" => profile.capabilities.multipoint && profile.multipoint.is_some(),
                _ => false,
            };
            if !capability_enabled || !experimental.insert(feature) {
                return Err(format!(
                    "invalid experimental feature {feature} in {}",
                    profile.id
                ));
            }
        }
        if profile.schema_version != 2 {
            return Err(format!("unsupported profile schema in {}", profile.id));
        }
        if profile.connection.is_none() {
            return Err(format!("missing connection profile in {}", profile.id));
        }
        if let Some(connection) = &profile.connection {
            if connection.provenance.trim().is_empty() {
                return Err(format!("missing transport provenance in {}", profile.id));
            }
            for uuid in [
                &connection.service_uuid,
                &connection.write_uuid,
                &connection.notify_uuid,
            ]
            .into_iter()
            .flatten()
            {
                uuid::Uuid::parse_str(uuid)
                    .map_err(|_| format!("invalid transport UUID in {}", profile.id))?;
            }
            if connection.transport == ControlTransport::BleGatt
                && (connection.service_uuid.is_none()
                    || connection.write_uuid.is_none()
                    || connection.notify_uuid.is_none()
                    || connection.framing == WireFraming::Unresolved)
            {
                return Err(format!(
                    "incomplete BLE connection profile in {}",
                    profile.id
                ));
            }
            if connection.transport == ControlTransport::Unresolved
                && (profile.support != "scanOnly"
                    || !connection.handshake.is_empty()
                    || connection.init_state_query)
            {
                return Err(format!(
                    "unresolved transport must remain passive in {}",
                    profile.id
                ));
            }
        }
        if !ids.insert(profile.id.clone()) {
            return Err(format!("duplicate model profile: {}", profile.id));
        }
        if !matches!(profile.protocol_family.as_str(), "bp1" | "unknown") {
            return Err(format!("unknown protocol family in {}", profile.id));
        }
        if !matches!(
            profile.support.as_str(),
            "verified" | "experimental" | "scanOnly"
        ) {
            return Err(format!("invalid support level in {}", profile.id));
        }
        if profile.id.is_empty() || profile.aliases.is_empty() {
            return Err("profiles must define an id and aliases".into());
        }
        if let Some(eq) = &profile.eq {
            if eq.bands.is_empty() || eq.min_gain >= eq.max_gain {
                return Err(format!("invalid EQ bands or gain limits in {}", profile.id));
            }
            let mut sorts = std::collections::HashSet::new();
            for preset in &eq.presets {
                if !preset.curve.is_empty() && preset.curve.len() != eq.bands.len() {
                    return Err(format!("EQ curve length mismatch in {}", profile.id));
                }
                if preset.filters.len() > 16
                    || preset.filters.iter().any(|filter| {
                        filter.frequency == 0
                            || !filter.q_value.is_finite()
                            || filter.q_value <= 0.0
                            || !filter.gain.is_finite()
                            || filter.gain < eq.min_gain
                            || filter.gain > eq.max_gain
                            || filter.filter > 2
                    })
                {
                    return Err(format!("invalid EQ filter payload in {}", profile.id));
                }
                if !sorts.insert(preset.dict_sort) {
                    return Err(format!("duplicate EQ dictSort in {}", profile.id));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_rejects_missing_invalid_or_active_unresolved_transport() {
        let mut profile = profile_for("bass-bp1-pro").unwrap();
        assert_eq!(profile.schema_version, 2);
        profile.connection = None;
        assert!(validate_profiles(&[profile]).is_err());
        let mut profile = profile_for("bass-bp1-pro").unwrap();
        profile.connection.as_mut().unwrap().notify_uuid = Some("not-a-uuid".into());
        assert!(validate_profiles(&[profile]).is_err());
        let mut profile = profile_for("bass-bp1-ultra").unwrap();
        assert_eq!(profile.support, "scanOnly");
        profile.connection.as_mut().unwrap().handshake = vec![0xBA, 5, 0];
        assert!(validate_profiles(&[profile]).is_err());
    }

    #[test]
    fn reviewed_pro_uuid_and_ultra_transport_are_not_inferred() {
        let pro = profile_for("bass-bp1-pro").unwrap().connection.unwrap();
        assert_eq!(pro.transport, ControlTransport::BleGatt);
        assert_eq!(pro.framing, WireFraming::BareAaBa);
        assert_eq!(
            pro.notify_uuid.as_deref(),
            Some("654b749c-e37f-ae1f-ebab-40ca133e3690")
        );
        let ultra = profile_for("bass-bp1-ultra").unwrap().connection.unwrap();
        assert_eq!(ultra.transport, ControlTransport::Unresolved);
        assert!(ultra.handshake.is_empty());
        assert!(!ultra.init_state_query);
    }

    #[test]
    fn bp1_profile_is_model_data_not_protocol_code() {
        let profile = profile_for("bass-bp1-pro").unwrap();
        assert_eq!(profile.protocol_family, "bp1");
        assert_eq!(profile.noise.environments, vec![101, 102, 103, 108]);
        assert_eq!(profile.eq.unwrap().presets.len(), 7);
        validate().unwrap();
    }

    #[test]
    fn rejects_duplicate_models_and_mismatched_curves() {
        let profile = profile_for("bass-bp1-pro").unwrap();
        assert!(validate_profiles(&[profile.clone(), profile.clone()]).is_err());
        let mut invalid = profile;
        invalid.eq.as_mut().unwrap().presets[0].curve = vec![1.0];
        assert!(validate_profiles(&[invalid]).is_err());
    }

    #[test]
    fn rejects_unregistered_family_and_misspelled_capabilities() {
        let mut profile = profile_for("bass-bp1-pro").unwrap();
        profile.protocol_family = "typo".into();
        assert!(validate_profiles(std::slice::from_ref(&profile)).is_err());
        let source = serde_json::to_string(&profile)
            .unwrap()
            .replace("gameMode", "gameMod");
        assert!(serde_json::from_str::<ModelProfile>(&source).is_err());
    }
    #[test]
    fn hearing_capability_requires_explicit_valid_threshold_constraints() {
        let mut model = profile_for("bass-bp1-pro").unwrap();
        model.capabilities.hearing_protection = true;
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.hearing = Some(types::HearingProfile {
            thresholds: vec![75, 85, 100],
            preserve_threshold_sentinel: false,
            provenance: "synthetic validation fixture".into(),
        });
        assert!(validate_profiles(&[model.clone()]).is_ok());
        model.hearing.as_mut().unwrap().thresholds = vec![1];
        assert!(validate_profiles(&[model]).is_err());
    }

    #[test]
    fn gesture_and_in_ear_capabilities_require_valid_source_constraints() {
        let mut model = profile_for("bass-bp1-pro").unwrap();
        model.capabilities.gesture = true;
        model.gesture = None;
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.gesture = Some(types::GestureProfile {
            dual_button: true,
            layouts: vec![types::GestureLayoutProfile {
                layout: 0,
                functions: vec![1, 0],
            }],
            provenance: "synthetic validation fixture".into(),
        });
        assert!(validate_profiles(&[model.clone()]).is_ok());
        model
            .gesture
            .as_mut()
            .unwrap()
            .layouts
            .push(types::GestureLayoutProfile {
                layout: 0,
                functions: vec![1],
            });
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.gesture.as_mut().unwrap().layouts = vec![types::GestureLayoutProfile {
            layout: 9,
            functions: vec![1],
        }];
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.gesture.as_mut().unwrap().layouts = vec![types::GestureLayoutProfile {
            layout: 0,
            functions: vec![99],
        }];
        assert!(validate_profiles(&[model.clone()]).is_err());

        let mut model = profile_for("bass-bp1-pro").unwrap();
        model.capabilities.in_ear = true;
        model.in_ear = None;
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.in_ear = Some(types::InEarProfile {
            provenance: "synthetic validation fixture".into(),
        });
        assert!(validate_profiles(&[model]).is_ok());
    }

    #[test]
    fn experimental_features_must_be_enabled_reviewed_capabilities() {
        let mut model = profile_for("bass-bp1-pro").unwrap();
        model.capabilities.gesture = false;
        model.experimental_features = vec!["gesture".into()];
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.capabilities.gesture = true;
        assert!(validate_profiles(&[model.clone()]).is_ok());
        model.experimental_features = vec!["gesture".into(), "gesture".into()];
        assert!(validate_profiles(&[model.clone()]).is_err());
        model.capabilities.gesture = false;
        model.experimental_features = vec!["rawOpcode".into()];
        assert!(validate_profiles(&[model]).is_err());
    }
}
