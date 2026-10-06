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
pub struct HearingReading {
    pub enabled: bool,
    pub level: u8,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InEarReading {
    pub enabled: bool,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipointReading {
    pub enabled: bool,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GestureReading {
    pub layout: u8,
    pub left: u8,
    pub right: u8,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncReading {
    pub mode: AncMode,
    pub parameter: u8,
    pub observed_at_ms: u64,
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
    pub anc: Option<AncReading>,
    pub eq: Option<EqPreset>,
    pub eq_index: Option<u8>,
    pub game: Option<bool>,
    pub ldac: Option<bool>,
    pub spatial_enabled: Option<bool>,
    pub bass_boost: Option<u8>,
    pub hearing: Option<HearingReading>,
    pub in_ear: Option<InEarReading>,
    pub multipoint: Option<MultipointReading>,
    pub gesture: Vec<GestureReading>,
}

impl DeviceSnapshot {
    pub fn new(session_id: u64) -> Self {
        Self {
            schema_version: 2,
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
            spatial_enabled: None,
            bass_boost: None,
            hearing: None,
            in_ear: None,
            multipoint: None,
            gesture: Vec::new(),
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
            DeviceEvent::Anc { mode, parameter } => {
                self.anc = Some(AncReading {
                    mode: *mode,
                    parameter: *parameter,
                    observed_at_ms: at_ms,
                })
            }
            DeviceEvent::Eq(value) => self.eq = Some(*value),
            DeviceEvent::EqIndex(value) => self.eq_index = Some(*value),
            DeviceEvent::GameMode(value) => self.game = Some(*value),
            DeviceEvent::Ldac(value) => self.ldac = Some(*value),
            DeviceEvent::SpatialEnabled(value) => self.spatial_enabled = Some(*value),
            DeviceEvent::BassBoost(value) => self.bass_boost = Some(*value),
            DeviceEvent::HearingProtection { enabled, level } => {
                self.hearing = Some(HearingReading {
                    enabled: *enabled,
                    level: *level,
                    observed_at_ms: at_ms,
                })
            }
            DeviceEvent::InEar(enabled) => {
                self.in_ear = Some(InEarReading {
                    enabled: *enabled,
                    observed_at_ms: at_ms,
                })
            }
            DeviceEvent::Multipoint(enabled) => {
                self.multipoint = Some(MultipointReading {
                    enabled: *enabled,
                    observed_at_ms: at_ms,
                })
            }
            DeviceEvent::GestureConfig {
                layout,
                left,
                right,
            } => {
                let reading = GestureReading {
                    layout: *layout,
                    left: *left,
                    right: *right,
                    observed_at_ms: at_ms,
                };
                match self
                    .gesture
                    .iter_mut()
                    .find(|existing| existing.layout == *layout)
                {
                    Some(existing) => *existing = reading,
                    None => self.gesture.push(reading),
                }
            }
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
    #[test]
    fn advanced_audio_stays_unknown_until_observed_and_resets_with_session() {
        let mut snapshot = DeviceSnapshot::new(1);
        assert!(snapshot.bass_boost.is_none() && snapshot.hearing.is_none());
        snapshot.observe(0x54, &DeviceEvent::BassBoost(0), 10);
        snapshot.observe(
            0x93,
            &DeviceEvent::HearingProtection {
                enabled: false,
                level: 0,
            },
            20,
        );
        assert_eq!(snapshot.bass_boost, Some(0));
        let hearing = snapshot.hearing.as_ref().unwrap();
        assert!(!hearing.enabled);
        assert_eq!(hearing.observed_at_ms, 20);
        snapshot.observe(
            0x93,
            &DeviceEvent::HearingProtection {
                enabled: false,
                level: 0,
            },
            30,
        );
        assert_eq!(snapshot.revision, 3);
        assert_eq!(snapshot.hearing.as_ref().unwrap().observed_at_ms, 30);
        let next = DeviceSnapshot::new(2);
        assert!(next.bass_boost.is_none() && next.hearing.is_none());
    }

    #[test]
    fn anc_snapshot_keeps_confirmed_parameter_and_observation_time() {
        let mut snapshot = DeviceSnapshot::new(4);
        assert!(snapshot.anc.is_none());

        snapshot.observe(
            0x34,
            &DeviceEvent::Anc {
                mode: AncMode::Anc,
                parameter: 103,
            },
            42,
        );

        let reading = snapshot.anc.as_ref().unwrap();
        assert_eq!(reading.mode, AncMode::Anc);
        assert_eq!(reading.parameter, 103);
        assert_eq!(reading.observed_at_ms, 42);

        let encoded = serde_json::to_value(snapshot).unwrap();
        assert_eq!(encoded["schemaVersion"], 2);
        assert_eq!(encoded["anc"]["mode"], "anc");
        assert_eq!(encoded["anc"]["parameter"], 103);
        assert_eq!(encoded["anc"]["observedAtMs"], 42);
    }

    #[test]
    fn in_ear_and_gesture_observations_replace_by_layout_and_keep_time() {
        let mut snapshot = DeviceSnapshot::new(5);
        assert!(snapshot.in_ear.is_none());
        assert!(snapshot.gesture.is_empty());

        snapshot.observe(0x25, &DeviceEvent::InEar(true), 10);
        snapshot.observe(
            0x21,
            &DeviceEvent::GestureConfig {
                layout: 3,
                left: 1,
                right: 1,
            },
            11,
        );
        snapshot.observe(
            0x21,
            &DeviceEvent::GestureConfig {
                layout: 0,
                left: 2,
                right: 3,
            },
            12,
        );
        snapshot.observe(
            0x21,
            &DeviceEvent::GestureConfig {
                layout: 3,
                left: 0,
                right: 0,
            },
            13,
        );

        assert!(snapshot.in_ear.as_ref().unwrap().enabled);
        assert_eq!(snapshot.in_ear.as_ref().unwrap().observed_at_ms, 10);
        assert_eq!(snapshot.gesture.len(), 2);
        let double = snapshot
            .gesture
            .iter()
            .find(|reading| reading.layout == 0)
            .unwrap();
        assert_eq!((double.left, double.right), (2, 3));
        let single = snapshot
            .gesture
            .iter()
            .find(|reading| reading.layout == 3)
            .unwrap();
        assert_eq!((single.left, single.right), (0, 0));
        assert_eq!(single.observed_at_ms, 13);

        let encoded = serde_json::to_value(snapshot).unwrap();
        assert_eq!(encoded["inEar"]["enabled"], true);
        assert_eq!(encoded["gesture"][0]["layout"], 3);
    }

    #[test]
    fn multipoint_observation_is_timestamped_and_resets_with_session() {
        let mut snapshot = DeviceSnapshot::new(6);
        assert!(snapshot.multipoint.is_none());
        snapshot.observe(0x57, &DeviceEvent::Multipoint(true), 21);
        assert!(snapshot.multipoint.as_ref().unwrap().enabled);
        assert_eq!(snapshot.multipoint.as_ref().unwrap().observed_at_ms, 21);
        snapshot.observe(0x57, &DeviceEvent::Multipoint(false), 22);
        assert!(!snapshot.multipoint.as_ref().unwrap().enabled);
        assert_eq!(snapshot.multipoint.as_ref().unwrap().observed_at_ms, 22);
        assert!(DeviceSnapshot::new(7).multipoint.is_none());
    }
}
