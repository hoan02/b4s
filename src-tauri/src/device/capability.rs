//! Backend authorization for feature intent. Public catalog metadata and
//! marketing aliases cannot supply runtime capabilities.

use crate::{catalog::ControlTransport, protocol::DeviceProfile};

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
}

pub fn authorize(profile: &DeviceProfile, feature: Feature) -> Result<(), String> {
    let connection = profile
        .connection
        .as_ref()
        .ok_or("No reviewed transport profile")?;
    if connection.transport != ControlTransport::BleGatt {
        return Err("Control transport has not been verified for this model".into());
    }
    if !profile.verified {
        return Err(
            "Experimental control is disabled; model evidence must be reviewed first".into(),
        );
    }
    if !connection.firmware_versions.is_empty()
        && !profile
            .firmware
            .as_ref()
            .is_some_and(|version| connection.firmware_versions.contains(version))
    {
        return Err("Firmware does not match the reviewed profile".into());
    }
    let capability = &profile.capabilities;
    let enabled = match feature {
        Feature::Listening => capability.anc,
        Feature::Eq => capability.eq,
        Feature::CustomEq => capability.eq && capability.custom_eq,
        Feature::Game => capability.game_mode,
        Feature::Bass => capability.bass_boost,
        Feature::Spatial => capability.spatial,
        Feature::Ldac => capability.ldac,
        Feature::Hearing => capability.hearing_protection,
        Feature::Find => capability.find_buds,
    };
    if enabled {
        Ok(())
    } else {
        Err(format!(
            "{feature:?} is not supported by this reviewed profile"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::profile_for;

    #[test]
    fn metadata_and_unsupported_feature_cannot_authorize_packets() {
        let pro = profile_for(Some("bass-bp1-pro"), None, None);
        assert!(authorize(&pro, Feature::Eq).is_ok());
        assert!(authorize(&pro, Feature::Ldac).is_err());
        assert!(authorize(&pro, Feature::Hearing).is_err());
        let ultra = profile_for(Some("bass-bp1-ultra"), None, None);
        assert!(authorize(&ultra, Feature::Eq).is_err());
        let legacy = profile_for(Some("eh10-nc-lite"), None, None);
        assert!(authorize(&legacy, Feature::Listening).is_err());
    }

    #[test]
    fn explicit_firmware_scope_rejects_unknown_and_different_versions() {
        let mut pro = profile_for(Some("bass-bp1-pro"), None, None);
        pro.connection.as_mut().unwrap().firmware_versions = vec!["test-firmware".into()];
        assert!(authorize(&pro, Feature::Eq).is_err());
        pro.firmware = Some("other".into());
        assert!(authorize(&pro, Feature::Eq).is_err());
        pro.firmware = Some("test-firmware".into());
        assert!(authorize(&pro, Feature::Eq).is_ok());
    }
}
