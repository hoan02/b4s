//! Match state evidence separately from transport completion and write ACKs.
//! AA/BA has no request ID: this proves observed state in the current session,
//! not that a particular write caused the state.

use super::session::SessionToken;
use crate::protocol::{AncMode, DeviceEvent, EqPreset};
use futures::future::BoxFuture;

#[derive(Clone, Debug)]
pub struct StateObservation {
    pub session: SessionToken,
    pub opcode: u8,
    pub event: DeviceEvent,
}

pub enum ExpectedState {
    Battery,
    Anc {
        mode: AncMode,
        parameter: u8,
    },
    Eq(EqPreset),
    EqIndex(u8),
    Game(bool),
    Bass(u8),
    SpatialMode(crate::protocol::SpatialMode),
    Ldac(bool),
    Hearing {
        enabled: bool,
        level: u8,
    },
    InEar(bool),
    Multipoint(bool),
    RestoreResult(u8),
    AdaptiveLr(bool),
    WindNoise(bool),
    Gesture {
        layout: u8,
        left: Option<u8>,
        right: Option<u8>,
    },
}

/// Transport seam for a write followed by a state query.
/// Implementations subscribe before the first write to avoid losing fast replies.
pub trait ConfirmedTransport {
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<StateObservation>;
    fn write<'a>(&'a self, payload: &'a [u8]) -> BoxFuture<'a, Result<(), String>>;
}

pub async fn write_and_confirm<T: ConfirmedTransport + ?Sized>(
    transport: &T,
    session: SessionToken,
    command: &[u8],
    query: &[u8],
    expected: ExpectedState,
) -> Result<StateObservation, String> {
    write_and_wait(transport, session, command, Some(query), expected).await
}

/// Wait for a state notification produced by a write without sending a
/// speculative query command. The observation subscription is established
/// before the write so a fast AA34 reply cannot be lost.
pub async fn write_and_observe<T: ConfirmedTransport + ?Sized>(
    transport: &T,
    session: SessionToken,
    command: &[u8],
    expected: ExpectedState,
) -> Result<StateObservation, String> {
    write_and_wait(transport, session, command, None, expected).await
}

async fn write_and_wait<T: ConfirmedTransport + ?Sized>(
    transport: &T,
    session: SessionToken,
    command: &[u8],
    query: Option<&[u8]>,
    expected: ExpectedState,
) -> Result<StateObservation, String> {
    let mut replies = transport.subscribe();
    transport.write(command).await?;
    if let Some(query) = query {
        transport.write(query).await?;
    }
    await_state(&mut replies, session, expected).await
}

impl ExpectedState {
    pub fn matches(&self, session: SessionToken, observation: &StateObservation) -> bool {
        if session != observation.session {
            return false;
        }
        match (self, observation.opcode, &observation.event) {
            (Self::Battery, 0x02, DeviceEvent::Battery(_)) => true,
            // AA34 is only the write status; noise state comes from the AA33 readback.
            (
                Self::Anc { mode, parameter },
                0x33,
                DeviceEvent::Anc {
                    mode: actual_mode,
                    parameter: actual_parameter,
                },
            ) => mode == actual_mode && parameter == actual_parameter,
            (Self::Eq(expected), 0x30, DeviceEvent::EqIndex(actual)) => {
                expected.to_byte() == *actual
            }
            (Self::EqIndex(expected), 0x30, DeviceEvent::EqIndex(actual)) => expected == actual,
            (Self::SpatialMode(expected), 0x42, DeviceEvent::SpatialMode(actual)) => {
                expected == actual
            }
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
            (Self::InEar(expected), 0x25, DeviceEvent::InEar(actual)) => expected == actual,
            (Self::Multipoint(expected), 0x57, DeviceEvent::Multipoint(actual)) => {
                expected == actual
            }
            (Self::RestoreResult(expected), 0x37, DeviceEvent::RestoreResult(actual)) => {
                expected == actual
            }
            (Self::AdaptiveLr(expected), 0x3F, DeviceEvent::AdaptiveLr(actual)) => {
                expected == actual
            }
            (Self::WindNoise(expected), 0x51, DeviceEvent::WindNoise(actual)) => expected == actual,
            (
                Self::Gesture {
                    layout,
                    left,
                    right,
                },
                0x21,
                DeviceEvent::GestureConfig {
                    layout: actual_layout,
                    left: actual_left,
                    right: actual_right,
                },
            ) => {
                layout == actual_layout
                    && left.is_none_or(|value| value == *actual_left)
                    && right.is_none_or(|value| value == *actual_right)
            }
            _ => false,
        }
    }

    /// Device-side rejection code for this transaction, if the observation is
    /// the write result and it is not success (`01` in every app consumer).
    pub fn rejection(&self, session: SessionToken, observation: &StateObservation) -> Option<u8> {
        if session != observation.session {
            return None;
        }
        let code = match (self, observation.opcode, &observation.event) {
            (Self::Anc { .. }, 0x34, DeviceEvent::WriteStatus { code, .. })
            | (Self::WindNoise(_), 0x52, DeviceEvent::WriteStatus { code, .. })
            | (Self::RestoreResult(_), 0x37, DeviceEvent::RestoreResult(code)) => *code,
            _ => return None,
        };
        (code != RESULT_SUCCESS).then_some(code)
    }
}

/// Success code shared by AA34, AA37 and AA52 (NoiseReduceManger.s,
/// EarPhoneSettingActivity receiver, HeadPhoneMainActivity.ResultHandle.i).
pub const RESULT_SUCCESS: u8 = 0x01;

/// Stable marker the frontend maps to a localized EarSetFailedUtil reason.
pub fn rejection_message(code: u8) -> String {
    format!("Device rejected the command [device-code:{code:02X}]")
}

/// Caller owns the transaction deadline/cancellation and subscribes before TX.
pub async fn await_state(
    replies: &mut tokio::sync::broadcast::Receiver<StateObservation>,
    session: SessionToken,
    expected: ExpectedState,
) -> Result<StateObservation, String> {
    loop {
        let observation = replies
            .recv()
            .await
            .map_err(|error| format!("State readback lost: {error}"))?;
        if let Some(code) = expected.rejection(session, &observation) {
            return Err(rejection_message(code));
        }
        if expected.matches(session, &observation) {
            return Ok(observation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::session::SessionEpoch;
    use crate::protocol::BatteryState;
    use std::{collections::VecDeque, sync::Mutex, time::Duration};

    enum ScriptedWrite {
        NoReply,
        Reply(StateObservation),
        DelayedReply(StateObservation, Duration),
        Disconnect,
        Fail(String),
    }

    struct FakeTransport {
        observations: Mutex<Option<tokio::sync::broadcast::Sender<StateObservation>>>,
        writes: Mutex<Vec<Vec<u8>>>,
        script: Mutex<VecDeque<ScriptedWrite>>,
    }

    impl FakeTransport {
        fn new(script: impl IntoIterator<Item = ScriptedWrite>) -> Self {
            Self {
                observations: Mutex::new(Some(tokio::sync::broadcast::channel(8).0)),
                writes: Mutex::new(Vec::new()),
                script: Mutex::new(script.into_iter().collect()),
            }
        }

        fn push(&self, script: impl IntoIterator<Item = ScriptedWrite>) {
            self.script.lock().unwrap().extend(script);
        }
    }

    impl ConfirmedTransport for FakeTransport {
        fn subscribe(&self) -> tokio::sync::broadcast::Receiver<StateObservation> {
            self.observations
                .lock()
                .unwrap()
                .as_ref()
                .expect("transport is connected")
                .subscribe()
        }

        fn write<'a>(&'a self, payload: &'a [u8]) -> BoxFuture<'a, Result<(), String>> {
            let step = self.script.lock().unwrap().pop_front();
            Box::pin(async move {
                self.writes.lock().unwrap().push(payload.to_vec());
                match step.unwrap_or(ScriptedWrite::NoReply) {
                    ScriptedWrite::NoReply => Ok(()),
                    ScriptedWrite::Reply(observation) => {
                        if let Some(sender) = self.observations.lock().unwrap().as_ref() {
                            let _ = sender.send(observation);
                        }
                        Ok(())
                    }
                    ScriptedWrite::DelayedReply(observation, delay) => {
                        let sender = self
                            .observations
                            .lock()
                            .unwrap()
                            .as_ref()
                            .cloned()
                            .ok_or_else(|| "Disconnected".to_owned())?;
                        tokio::spawn(async move {
                            tokio::time::sleep(delay).await;
                            let _ = sender.send(observation);
                        });
                        Ok(())
                    }
                    ScriptedWrite::Disconnect => {
                        self.observations.lock().unwrap().take();
                        Ok(())
                    }
                    ScriptedWrite::Fail(error) => Err(error),
                }
            })
        }
    }

    fn bass_observation(session: SessionToken, opcode: u8, value: u8) -> StateObservation {
        StateObservation {
            session,
            opcode,
            event: DeviceEvent::BassBoost(value),
        }
    }

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
    fn spatial_confirmation_requires_the_exact_selected_mode_from_aa42() {
        use crate::protocol::SpatialMode;
        let mut epoch = SessionEpoch::default();
        let session = epoch.token();
        let mut observation = StateObservation {
            session,
            opcode: 0x42,
            event: DeviceEvent::SpatialMode(SpatialMode::Music),
        };
        assert!(ExpectedState::SpatialMode(SpatialMode::Music).matches(session, &observation));
        assert!(!ExpectedState::SpatialMode(SpatialMode::Cinema).matches(session, &observation));
        // AA43 is the write acknowledgement, never state.
        observation.opcode = 0x43;
        assert!(!ExpectedState::SpatialMode(SpatialMode::Music).matches(session, &observation));
        observation.opcode = 0x42;
        epoch.invalidate();
        assert!(
            !ExpectedState::SpatialMode(SpatialMode::Music).matches(epoch.token(), &observation)
        );
    }

    #[tokio::test]
    async fn scripted_observations_skip_wrong_state_and_session_and_fail_on_loss() {
        let mut epoch = SessionEpoch::default();
        let old = epoch.token();
        epoch.invalidate();
        let session = epoch.token();
        let (tx, mut rx) = tokio::sync::broadcast::channel(8);
        for (token, opcode, value) in [
            (old, 0x53, 1),
            (session, 0x54, 1),
            (session, 0x53, 0),
            (session, 0x53, 1),
        ] {
            tx.send(StateObservation {
                session: token,
                opcode,
                event: DeviceEvent::BassBoost(value),
            })
            .unwrap();
        }
        let found = await_state(&mut rx, session, ExpectedState::Bass(1))
            .await
            .unwrap();
        assert_eq!(found.opcode, 0x53);
        drop(tx);
        assert!(await_state(&mut rx, session, ExpectedState::Bass(1))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn lagged_readback_fails_instead_of_confirming_from_remaining_messages() {
        let session = SessionEpoch::default().token();
        let (tx, mut rx) = tokio::sync::broadcast::channel(1);
        for _ in 0..3 {
            tx.send(StateObservation {
                session,
                opcode: 0x53,
                event: DeviceEvent::BassBoost(1),
            })
            .unwrap();
        }
        assert!(await_state(&mut rx, session, ExpectedState::Bass(1))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn scripted_transport_writes_query_and_ignores_ack_until_readback() {
        let session = SessionEpoch::default().token();
        let transport = FakeTransport::new([
            ScriptedWrite::Reply(bass_observation(session, 0x54, 1)),
            ScriptedWrite::Reply(bass_observation(session, 0x53, 1)),
        ]);

        let found = write_and_confirm(
            &transport,
            session,
            &[0xBA, 0x54, 1],
            &[0xBA, 0x53],
            ExpectedState::Bass(1),
        )
        .await
        .unwrap();

        assert_eq!(found.opcode, 0x53);
        assert_eq!(
            *transport.writes.lock().unwrap(),
            vec![vec![0xBA, 0x54, 1], vec![0xBA, 0x53]]
        );
    }

    fn status(session: SessionToken, opcode: u8, code: u8) -> StateObservation {
        StateObservation {
            session,
            opcode,
            event: DeviceEvent::WriteStatus { opcode, code },
        }
    }

    #[tokio::test]
    async fn anc_write_confirms_only_from_the_aa33_readback_after_the_status() {
        let session = SessionEpoch::default().token();
        let readback = StateObservation {
            session,
            opcode: 0x33,
            event: DeviceEvent::Anc {
                mode: AncMode::Transparency,
                parameter: 0xFF,
            },
        };
        let transport = FakeTransport::new([
            ScriptedWrite::Reply(status(session, 0x34, 0x01)),
            ScriptedWrite::Reply(readback),
        ]);
        let found = write_and_confirm(
            &transport,
            session,
            &[0xBA, 0x34, 0x02, 0xFF],
            &[0xBA, 0x33],
            ExpectedState::Anc {
                mode: AncMode::Transparency,
                parameter: 0xFF,
            },
        )
        .await
        .unwrap();
        assert_eq!(found.opcode, 0x33);
        assert!(!ExpectedState::Anc {
            mode: AncMode::Transparency,
            parameter: 1,
        }
        .matches(session, &found));
        assert_eq!(
            *transport.writes.lock().unwrap(),
            vec![vec![0xBA, 0x34, 0x02, 0xFF], vec![0xBA, 0x33]]
        );
    }

    #[tokio::test]
    async fn rejected_write_status_fails_fast_with_the_device_code() {
        let session = SessionEpoch::default().token();
        let transport = FakeTransport::new([ScriptedWrite::Reply(status(session, 0x34, 0x0C))]);
        let error = write_and_confirm(
            &transport,
            session,
            &[0xBA, 0x34, 0x01, 0x68],
            &[0xBA, 0x33],
            ExpectedState::Anc {
                mode: AncMode::Anc,
                parameter: 0x68,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error, rejection_message(0x0C));
        assert!(error.contains("[device-code:0C]"));

        let wind = ExpectedState::WindNoise(true);
        assert_eq!(
            wind.rejection(session, &status(session, 0x52, 0x0A)),
            Some(0x0A)
        );
        assert_eq!(wind.rejection(session, &status(session, 0x52, 0x01)), None);
        // A status for another opcode or session never rejects this write.
        assert_eq!(wind.rejection(session, &status(session, 0x34, 0x00)), None);
        let mut epoch = SessionEpoch::default();
        let old = epoch.token();
        epoch.invalidate();
        assert_eq!(
            wind.rejection(epoch.token(), &status(old, 0x52, 0x0A)),
            None
        );
    }

    #[tokio::test]
    async fn disconnect_and_write_failure_never_confirm_a_feature() {
        let session = SessionEpoch::default().token();
        let disconnected = FakeTransport::new([ScriptedWrite::Disconnect]);
        assert!(write_and_confirm(
            &disconnected,
            session,
            &[0xBA, 0x54, 1],
            &[0xBA, 0x53],
            ExpectedState::Bass(1),
        )
        .await
        .is_err());

        let failed = FakeTransport::new([ScriptedWrite::Fail("write failed".into())]);
        assert_eq!(
            write_and_confirm(
                &failed,
                session,
                &[0xBA, 0x54, 1],
                &[0xBA, 0x53],
                ExpectedState::Bass(1),
            )
            .await
            .unwrap_err(),
            "write failed"
        );
        assert_eq!(failed.writes.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn deadline_and_late_ack_do_not_confirm_or_poison_the_next_transaction() {
        let session = SessionEpoch::default().token();
        let transport = FakeTransport::new([
            ScriptedWrite::DelayedReply(
                bass_observation(session, 0x54, 1),
                Duration::from_millis(100),
            ),
            ScriptedWrite::NoReply,
        ]);
        let timed_out = tokio::time::timeout(
            Duration::from_millis(50),
            write_and_confirm(
                &transport,
                session,
                &[0xBA, 0x54, 1],
                &[0xBA, 0x53],
                ExpectedState::Bass(1),
            ),
        )
        .await;
        assert!(timed_out.is_err());

        tokio::time::sleep(Duration::from_millis(120)).await;
        transport.push([
            ScriptedWrite::NoReply,
            ScriptedWrite::DelayedReply(
                bass_observation(session, 0x53, 1),
                Duration::from_millis(1),
            ),
        ]);
        let found = write_and_confirm(
            &transport,
            session,
            &[0xBA, 0x54, 1],
            &[0xBA, 0x53],
            ExpectedState::Bass(1),
        )
        .await
        .unwrap();
        assert_eq!(found.opcode, 0x53);
    }

    #[test]
    fn gesture_and_in_ear_confirm_only_from_their_state_opcode() {
        let session = SessionEpoch::default().token();
        let mut multipoint = StateObservation {
            session,
            opcode: 0x57,
            event: DeviceEvent::Multipoint(true),
        };
        assert!(ExpectedState::Multipoint(true).matches(session, &multipoint));
        assert!(!ExpectedState::Multipoint(false).matches(session, &multipoint));
        multipoint.opcode = 0x58;
        assert!(!ExpectedState::Multipoint(true).matches(session, &multipoint));

        let mut restore = StateObservation {
            session,
            opcode: 0x37,
            event: DeviceEvent::RestoreResult(RESULT_SUCCESS),
        };
        let expected = ExpectedState::RestoreResult(RESULT_SUCCESS);
        assert!(expected.matches(session, &restore));
        assert_eq!(expected.rejection(session, &restore), None);
        restore.event = DeviceEvent::RestoreResult(0x0C);
        assert!(!expected.matches(session, &restore));
        assert_eq!(expected.rejection(session, &restore), Some(0x0C));
        // 00 is not success on the settings screens these models use.
        restore.event = DeviceEvent::RestoreResult(0x00);
        assert_eq!(expected.rejection(session, &restore), Some(0x00));
        restore.event = DeviceEvent::RestoreResult(RESULT_SUCCESS);
        restore.opcode = 0x36;
        assert!(!expected.matches(session, &restore));

        let mut adaptive = StateObservation {
            session,
            opcode: 0x3F,
            event: DeviceEvent::AdaptiveLr(true),
        };
        assert!(ExpectedState::AdaptiveLr(true).matches(session, &adaptive));
        assert!(!ExpectedState::AdaptiveLr(false).matches(session, &adaptive));
        adaptive.opcode = 0x4A;
        assert!(!ExpectedState::AdaptiveLr(true).matches(session, &adaptive));

        let mut wind = StateObservation {
            session,
            opcode: 0x51,
            event: DeviceEvent::WindNoise(true),
        };
        assert!(ExpectedState::WindNoise(true).matches(session, &wind));
        assert!(!ExpectedState::WindNoise(false).matches(session, &wind));
        wind.opcode = 0x52;
        assert!(!ExpectedState::WindNoise(true).matches(session, &wind));

        let mut observation = StateObservation {
            session,
            opcode: 0x25,
            event: DeviceEvent::InEar(true),
        };
        assert!(ExpectedState::InEar(true).matches(session, &observation));
        assert!(!ExpectedState::InEar(false).matches(session, &observation));
        // AA26 set reply is never the state opcode.
        observation.opcode = 0x26;
        assert!(!ExpectedState::InEar(true).matches(session, &observation));

        let mut observation = StateObservation {
            session,
            opcode: 0x21,
            event: DeviceEvent::GestureConfig {
                layout: 3,
                left: 1,
                right: 1,
            },
        };
        assert!(ExpectedState::Gesture {
            layout: 3,
            left: Some(1),
            right: Some(1),
        }
        .matches(session, &observation));
        // A None side accepts whatever the device reports (unchanged bud).
        assert!(ExpectedState::Gesture {
            layout: 3,
            left: Some(1),
            right: None,
        }
        .matches(session, &observation));
        assert!(!ExpectedState::Gesture {
            layout: 3,
            left: Some(0),
            right: Some(0),
        }
        .matches(session, &observation));
        observation.opcode = 0x22;
        assert!(!ExpectedState::Gesture {
            layout: 3,
            left: Some(1),
            right: Some(1),
        }
        .matches(session, &observation));
    }
}
