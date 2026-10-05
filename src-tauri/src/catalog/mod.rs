mod types;

pub use types::ModelProfile;

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
        if !ids.insert(profile.id.clone()) {
            return Err(format!("duplicate model profile: {}", profile.id));
        }
        if !matches!(
            profile.protocol_family.as_str(),
            "bp1" | "baseusAaBaExperimental" | "unknown"
        ) {
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
                if preset.curve.len() != eq.bands.len() {
                    return Err(format!("EQ curve length mismatch in {}", profile.id));
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
    fn bp1_profile_is_model_data_not_protocol_code() {
        let profile = profile_for("bass-bp1-pro").unwrap();
        assert_eq!(profile.protocol_family, "bp1");
        assert_eq!(profile.noise.environments, vec![101, 102, 103, 108]);
        assert_eq!(profile.eq.unwrap().presets.len(), 12);
        validate().unwrap();
    }

    #[test]
    fn rejects_duplicate_models_and_mismatched_curves() {
        let profile = profile_for("bass-bp1-pro").unwrap();
        assert!(validate_profiles(&[profile.clone(), profile.clone()]).is_err());
        let mut invalid = profile;
        invalid.eq.as_mut().unwrap().presets[0].curve.pop();
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
}
