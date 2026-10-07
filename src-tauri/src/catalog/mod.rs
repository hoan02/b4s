pub mod evidence;
pub mod features;
pub mod public;
mod types;
mod validation;
use validation::validate_profiles;

pub use types::Capabilities;
pub use types::ModelProfile;
pub use types::SoundProfile;
pub use types::{ConnectionProfile, ControlTransport, WireFraming};

include!(concat!(env!("OUT_DIR"), "/model_profiles.rs"));

static PROFILES: std::sync::LazyLock<Vec<ModelProfile>> = std::sync::LazyLock::new(|| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_public_headphone_has_an_explicit_runtime_profile() {
        let profiles = all_profiles();
        assert_eq!(profiles.len(), 124);
        for public in public::headphone_models() {
            let profile = profiles
                .iter()
                .find(|profile| {
                    public::identity_key(&profile.display_name)
                        == public::identity_key(&public.model)
                })
                .expect("public headphone must be embedded in the runtime catalog");
            if profile.support == "scanOnly" {
                assert_eq!(profile.support, "scanOnly");
                assert_eq!(profile.protocol_family, "unknown");
                assert_eq!(profile.capabilities, Capabilities::default());
                let connection = profile.connection.as_ref().unwrap();
                assert_eq!(connection.transport, ControlTransport::Unresolved);
                let runtime = crate::protocol::models::profile_for(Some(&profile.id), None, None);
                assert_eq!(runtime.connection.as_ref(), Some(connection));
                assert!(!runtime.verified);
            }
        }
    }

    #[test]
    fn model_contracts_reject_missing_limits_old_schema_and_unimplemented_gesture() {
        let pro = profile_for("bass-bp1-pro").unwrap();
        for mutation in 0..8 {
            let mut model = pro.clone();
            match mutation {
                0 => model.schema_version = 2,
                1 => model.sound = None,
                2 => model.sound.as_mut().unwrap().max_bass_level = 5,
                3 => model.eq.as_mut().unwrap().custom_write.slot = 100,
                4 => model.gesture.as_mut().unwrap().protocol = types::GestureProtocol::V2,
                5 => {
                    model.feature_evidence.remove("gestureV2");
                }
                6 => model.eq.as_mut().unwrap().bands[0] = 0,
                _ => {
                    model.eq.as_mut().unwrap().bands.pop();
                    model.eq.as_mut().unwrap().q_values.pop();
                }
            }
            assert!(validate_profiles(&[model]).is_err());
        }
    }

    #[test]
    fn enabled_features_require_implementation_evidence_without_claiming_firmware_coverage() {
        let mut pro = profile_for("bass-bp1-pro").unwrap();
        assert!(pro.feature_evidence["anc"].firmware_versions.is_empty());
        assert_eq!(
            pro.feature_evidence["gestureV2"].status,
            evidence::EvidenceStatus::Unknown
        );
        pro.feature_evidence.get_mut("anc").unwrap().status =
            evidence::EvidenceStatus::SourceReviewed;
        assert!(validate_profiles(&[pro]).is_err());
        let mut pro = profile_for("bass-bp1-pro").unwrap();
        pro.feature_evidence
            .get_mut("anc")
            .unwrap()
            .provenance
            .clear();
        assert!(validate_profiles(&[pro]).is_err());
    }

    #[test]
    fn v3_rejects_missing_invalid_or_active_unresolved_transport() {
        let mut profile = profile_for("bass-bp1-pro").unwrap();
        assert_eq!(profile.schema_version, 3);
        profile.connection = None;
        assert!(validate_profiles(&[profile]).is_err());
        let mut profile = profile_for("bass-bp1-pro").unwrap();
        profile.connection.as_mut().unwrap().notify_uuid = Some("not-a-uuid".into());
        assert!(validate_profiles(&[profile]).is_err());
        let mut profile = profile_for("bass-bp1-ultra").unwrap();
        profile.support = "scanOnly".into();
        profile.connection.as_mut().unwrap().transport = ControlTransport::Unresolved;
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
        assert_eq!(ultra.transport, ControlTransport::BleGatt);
        assert_eq!(ultra.framing, WireFraming::Headphone789c);
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
        model
            .feature_evidence
            .get_mut("hearingProtection")
            .unwrap()
            .status = evidence::EvidenceStatus::Implemented;
        model
            .feature_evidence
            .get_mut("hearingProtection")
            .unwrap()
            .provenance = "test fixture".into();
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
            protocol: types::GestureProtocol::Legacy,
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

    #[test]
    fn custom_eq_q_values_require_one_positive_finite_value_per_band() {
        let mut model = profile_for("bass-bp1-pro").unwrap();
        assert!(validate_profiles(&[model.clone()]).is_ok());
        for invalid in [
            vec![1.0],
            vec![0.0; 8],
            vec![f32::NAN; 8],
            vec![f32::INFINITY; 8],
        ] {
            model.eq.as_mut().unwrap().q_values = invalid;
            assert!(validate_profiles(&[model.clone()]).is_err());
        }
        model.eq.as_mut().unwrap().q_values.clear();
        assert!(validate_profiles(&[model]).is_err());
    }
}
