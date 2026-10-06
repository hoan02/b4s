use crate::protocol::{Command, DeviceProfile, ModelInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupQuery {
    Battery,
    Eq,
    Bass,
    Spatial,
    Ldac,
    HearingProtection,
    InEar,
    Multipoint,
    RestoreSupport,
    AdaptiveLr,
    Gesture(u8),
}

pub fn plan_for(model: Option<&ModelInfo>, profile: &DeviceProfile) -> Vec<StartupQuery> {
    let mut plan = Vec::new();
    let Some(model) = model else {
        return plan;
    };
    if profile.model_id.as_deref() != Some(model.id.as_str())
        || profile.protocol == crate::protocol::ProtocolFamily::Unknown
        || super::capability::authorize_control(profile).is_err()
    {
        return plan;
    }
    use super::capability::{authorize, Feature};
    plan.push(StartupQuery::Battery);
    for (feature, query) in [
        (Feature::Eq, StartupQuery::Eq),
        (Feature::Bass, StartupQuery::Bass),
        (Feature::Spatial, StartupQuery::Spatial),
        (Feature::Ldac, StartupQuery::Ldac),
        (Feature::Hearing, StartupQuery::HearingProtection),
        (Feature::InEar, StartupQuery::InEar),
        (Feature::Multipoint, StartupQuery::Multipoint),
        (Feature::RestoreDefaults, StartupQuery::RestoreSupport),
        (Feature::AdaptiveLr, StartupQuery::AdaptiveLr),
    ] {
        if authorize(profile, feature).is_ok() {
            plan.push(query);
        }
    }
    // Gesture v1 state is queried per reviewed layout once the capability and
    // the Experimental gate allow it.
    if authorize(profile, Feature::Gesture).is_ok() {
        if let Some(gesture) = profile
            .model_id
            .as_deref()
            .and_then(crate::catalog::profile_for)
            .and_then(|model| model.gesture)
        {
            for layout in gesture.layouts {
                plan.push(StartupQuery::Gesture(layout.layout));
            }
        }
    }
    plan
}

pub fn command_for(query: StartupQuery) -> Option<Command> {
    match query {
        StartupQuery::Battery => Some(Command::QueryBattery),
        StartupQuery::Eq => Some(Command::QueryEq),
        StartupQuery::Bass => Some(Command::QueryBassBoost),
        StartupQuery::Spatial => Some(Command::QuerySpatial),
        StartupQuery::Ldac => Some(Command::QueryLdac),
        StartupQuery::HearingProtection => Some(Command::QueryHearingProtection),
        StartupQuery::InEar => Some(Command::QueryInEar),
        StartupQuery::Multipoint => Some(Command::QueryMultipoint),
        StartupQuery::RestoreSupport => Some(Command::QueryRestoreSupport),
        StartupQuery::AdaptiveLr => Some(Command::QueryAdaptiveLr),
        StartupQuery::Gesture(layout) => Some(Command::QueryGesture(layout)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{catalog_json, profile_for};

    #[test]
    fn verified_bp1_queries_declared_capabilities() {
        let model = catalog_json()
            .into_iter()
            .find(|m| m.id == "bass-bp1-pro")
            .unwrap();
        let mut profile = profile_for(Some(&model.id), None, None);
        // Clear the Experimental gate so the planner is deterministic: every
        // enabled capability is queried, including the experimental in-ear one.
        profile.experimental_features.clear();
        assert_eq!(
            plan_for(Some(&model), &profile),
            vec![
                StartupQuery::Battery,
                StartupQuery::Eq,
                StartupQuery::Bass,
                StartupQuery::Spatial,
                StartupQuery::InEar,
                StartupQuery::Multipoint,
                StartupQuery::RestoreSupport,
                StartupQuery::AdaptiveLr,
                StartupQuery::Gesture(0),
                StartupQuery::Gesture(1),
                StartupQuery::Gesture(2),
                StartupQuery::Gesture(3)
            ]
        );
    }

    #[test]
    fn unknown_device_gets_no_speculative_query() {
        let profile = profile_for(None, None, None);
        assert!(plan_for(None, &profile).is_empty());
    }

    #[test]
    fn query_maps_to_existing_wire_command() {
        assert!(matches!(
            command_for(StartupQuery::Eq),
            Some(Command::QueryEq)
        ));
    }
    #[test]
    fn startup_ignores_marketing_flags_and_rejects_firmware_and_identity_mismatch() {
        let mut model = catalog_json()
            .into_iter()
            .find(|m| m.id == "bass-bp1-pro")
            .unwrap();
        let mut profile = profile_for(Some(&model.id), None, None);
        model.capabilities.ldac = true;
        model.capabilities.hearing_protection = true;
        assert!(!plan_for(Some(&model), &profile).contains(&StartupQuery::Ldac));
        assert!(!plan_for(Some(&model), &profile).contains(&StartupQuery::HearingProtection));
        profile.connection.as_mut().unwrap().firmware_versions = vec!["reviewed-version".into()];
        assert!(plan_for(Some(&model), &profile).is_empty());
        profile
            .connection
            .as_mut()
            .unwrap()
            .firmware_versions
            .clear();
        model.id = "different-model".into();
        assert!(plan_for(Some(&model), &profile).is_empty());
        assert_eq!(
            crate::protocol::encode_command(Command::QueryBassBoost),
            vec![0xBA, 0x53]
        );
    }
}
