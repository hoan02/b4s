mod types;

pub use types::ModelProfile;

const BP1_PROFILE: &str = include_str!("../../catalog/models/bass-bp1-pro.json");

pub fn all_profiles() -> Vec<ModelProfile> {
    vec![serde_json::from_str(BP1_PROFILE).expect("valid bass-bp1-pro profile")]
}

pub fn profile_for(model_id: &str) -> Option<ModelProfile> {
    all_profiles().into_iter().find(|profile| profile.id == model_id)
}

#[allow(dead_code)]
pub fn validate() -> Result<(), String> {
    let profiles = all_profiles();
    let mut ids = std::collections::HashSet::new();
    for profile in profiles {
        if !ids.insert(profile.id.clone()) {
            return Err(format!("duplicate model profile: {}", profile.id));
        }
        if let Some(eq) = profile.eq {
            if eq.bands.len() != 8 {
                return Err(format!("{} must define eight EQ bands", profile.id));
            }
            let mut sorts = std::collections::HashSet::new();
            for preset in eq.presets {
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
}
