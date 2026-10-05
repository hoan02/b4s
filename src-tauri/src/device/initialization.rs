use crate::protocol::{Command, DeviceProfile, ModelInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupQuery {
    Battery,
    Eq,
    Ldac,
    HearingProtection,
}

pub fn plan_for(model: Option<&ModelInfo>, profile: &DeviceProfile) -> Vec<StartupQuery> {
    let mut plan = Vec::new();
    let Some(model) = model else {
        return plan;
    };
    plan.push(StartupQuery::Battery);
    if model.capabilities.eq {
        plan.push(StartupQuery::Eq);
    }
    if model.capabilities.ldac {
        plan.push(StartupQuery::Ldac);
    }
    if model.capabilities.hearing_protection && profile.verified {
        plan.push(StartupQuery::HearingProtection);
    }
    plan
}

pub fn command_for(query: StartupQuery) -> Option<Command> {
    match query {
        StartupQuery::Battery => Some(Command::QueryBattery),
        StartupQuery::Eq => Some(Command::QueryEq),
        StartupQuery::Ldac => Some(Command::QueryLdac),
        StartupQuery::HearingProtection => Some(Command::QueryHearingProtection),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{catalog_json, profile_for};

    #[test]
    fn verified_bp1_queries_declared_capabilities() {
        let model = catalog_json().into_iter().find(|m| m.id == "bass-bp1-pro").unwrap();
        let profile = profile_for(Some(&model.id), None, None);
        assert_eq!(plan_for(Some(&model), &profile), vec![StartupQuery::Battery, StartupQuery::Eq]);
    }

    #[test]
    fn unknown_device_gets_no_speculative_query() {
        let profile = profile_for(None, None, None);
        assert!(plan_for(None, &profile).is_empty());
    }

    #[test]
    fn query_maps_to_existing_wire_command() {
        assert!(matches!(command_for(StartupQuery::Eq), Some(Command::QueryEq)));
    }
}
