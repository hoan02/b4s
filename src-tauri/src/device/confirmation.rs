//! Match state evidence separately from transport completion and write ACKs.
//! AA/BA has no request ID: this proves observed state in the current session,
//! not that a particular write caused the state.

use super::session::SessionToken;
use crate::protocol::{DeviceEvent, EqPreset};

#[derive(Clone, Debug)]
pub struct StateObservation {
    pub session: SessionToken,
    pub opcode: u8,
    pub event: DeviceEvent,
}

pub enum ExpectedState {
    Battery,
    Eq(EqPreset),
    EqIndex(u8),
    Game(bool),
    Bass(u8),
    SpatialEnabled(bool),
    Ldac(bool),
    Hearing { enabled: bool, level: u8 },
}

impl ExpectedState {
    pub fn matches(&self, session: SessionToken, observation: &StateObservation) -> bool {
        if session != observation.session {
            return false;
        }
        match (self, observation.opcode, &observation.event) {
            (Self::Battery, 0x02, DeviceEvent::Battery(_)) => true,
            (Self::Eq(expected), 0x30, DeviceEvent::EqIndex(actual)) => expected.to_byte() == *actual,
            (Self::EqIndex(expected), 0x30, DeviceEvent::EqIndex(actual)) => expected == actual,
            (Self::SpatialEnabled(expected), 0x42, DeviceEvent::SpatialEnabled(actual)) => expected == actual,
            (Self::Bass(expected), 0x53, DeviceEvent::BassBoost(actual)) => expected == actual,
            (Self::Game(expected), 0x23, DeviceEvent::GameMode(actual)) => expected == actual,
            (Self::Ldac(expected), 0x74, DeviceEvent::Ldac(actual)) => expected == actual,
            (
                Self::Hearing { enabled, level },
                0x93,
                DeviceEvent::HearingProtection {
                    enabled: actual,
                    level: actual_level,
                },
            ) => enabled == actual && level == actual_level,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::session::SessionEpoch;
    use crate::protocol::BatteryState;

    #[test]
    fn unchanged_value_readback_confirms_but_ack_and_other_state_do_not() {
        let epoch = SessionEpoch::default();
        let session = epoch.token();
        let mut observation = StateObservation {
            session,
            opcode: 0x30,
            event: DeviceEvent::EqIndex(0),
        };
        assert!(ExpectedState::Eq(EqPreset::Balanced).matches(session, &observation));
        observation.opcode = 0x43;
        assert!(!ExpectedState::Eq(EqPreset::Balanced).matches(session, &observation));
        assert!(!ExpectedState::Game(false).matches(session, &observation));
    }

    #[test]
    fn old_session_and_case_only_reply_cannot_finish_bud_query() {
        let mut epoch = SessionEpoch::default();
        let mut observation = StateObservation {
            session: epoch.token(),
            opcode: 0x02,
            event: DeviceEvent::Battery(BatteryState::default()),
        };
        assert!(ExpectedState::Battery.matches(epoch.token(), &observation));
        epoch.invalidate();
        assert!(!ExpectedState::Battery.matches(epoch.token(), &observation));
        observation.session = epoch.token();
        observation.opcode = 0x27;
        assert!(!ExpectedState::Battery.matches(epoch.token(), &observation));
    }
    #[test]
    fn spatial_enable_requires_query_state_not_ack_or_other_session() {
        let epoch = SessionEpoch::default();
        let session = epoch.token();
        let mut observation = StateObservation { session, opcode: 0x42,
            event: DeviceEvent::SpatialEnabled(true) };
        assert!(ExpectedState::SpatialEnabled(true).matches(session, &observation));
        assert!(!ExpectedState::SpatialEnabled(false).matches(session, &observation));
        observation.opcode = 0x43;
        assert!(!ExpectedState::SpatialEnabled(true).matches(session, &observation));
        observation.opcode = 0x42;
        epoch.invalidate();
        assert!(!ExpectedState::SpatialEnabled(true).matches(epoch.token(), &observation));
    }

}
