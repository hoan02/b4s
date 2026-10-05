use crate::protocol::{AncMode, DeviceEvent, EqPreset};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryReading {
    pub percentage: u8,
    pub charging: bool,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatterySnapshot {
    pub left: Option<BatteryReading>,
    pub right: Option<BatteryReading>,
    pub case: Option<BatteryReading>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub schema_version: u8,
    pub session_id: u64,
    pub revision: u64,
    pub model_id: Option<String>,
    pub device_id: Option<String>,
    pub mock: bool,
    pub battery: BatterySnapshot,
    pub anc: Option<AncMode>,
    pub eq: Option<EqPreset>,
    pub eq_index: Option<u8>,
    pub game: Option<bool>,
    pub ldac: Option<bool>,
}

impl DeviceSnapshot {
    pub fn new(session_id: u64) -> Self {
        Self {
            schema_version: 1,
            session_id,
            revision: 0,
            model_id: None,
            device_id: None,
            mock: false,
            battery: BatterySnapshot::default(),
            anc: None,
            eq: None,
            eq_index: None,
            game: None,
            ldac: None,
        }
    }

    pub fn observe(&mut self, opcode: u8, event: &DeviceEvent, at_ms: u64) {
        match event {
            DeviceEvent::Battery(battery) if opcode == 2 => {
                self.battery.left = Some(BatteryReading {
                    percentage: battery.left,
                    charging: battery.left_charging,
                    observed_at_ms: at_ms,
                });
                self.battery.right = Some(BatteryReading {
                    percentage: battery.right,
                    charging: battery.right_charging,
                    observed_at_ms: at_ms,
                });
            }
            DeviceEvent::Battery(battery) if opcode == 0x27 => {
                self.battery.case = Some(BatteryReading {
                    percentage: battery.case,
                    charging: battery.case_charging,
                    observed_at_ms: at_ms,
                });
            }
            DeviceEvent::Anc(value) => self.anc = Some(*value),
            DeviceEvent::Eq(value) => self.eq = Some(*value),
            DeviceEvent::EqIndex(value) => self.eq_index = Some(*value),
            DeviceEvent::GameMode(value) => self.game = Some(*value),
            DeviceEvent::Ldac(value) => self.ldac = Some(*value),
            _ => return,
        }
        // Receipt revision advances even when the observed value is unchanged.
        self.revision = self
            .revision
            .checked_add(1)
            .expect("snapshot revision exhausted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::BatteryState;

    #[test]
    fn unknown_zero_and_unchanged_receipt_are_distinct() {
        let mut snapshot = DeviceSnapshot::new(1);
        assert!(snapshot.battery.left.is_none());
        let event = DeviceEvent::Battery(BatteryState::default());
        snapshot.observe(2, &event, 10);
        assert_eq!(snapshot.battery.left.as_ref().unwrap().percentage, 0);
        assert!(snapshot.battery.case.is_none());
        snapshot.observe(2, &event, 20);
        assert_eq!(snapshot.revision, 2);
        assert_eq!(snapshot.battery.left.as_ref().unwrap().observed_at_ms, 20);
        snapshot.observe(0x27, &event, 30);
        assert_eq!(snapshot.battery.case.as_ref().unwrap().percentage, 0);
        assert!(DeviceSnapshot::new(2).battery.left.is_none());
    }
}
