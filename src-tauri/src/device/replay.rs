//! Synthetic pipeline replay; never hardware support evidence.
use super::{confirmation::{ExpectedState, StateObservation}, session::SessionEpoch};
use crate::protocol::{Bp1ProAnc, Frame, receiver::NotificationReceiver, wrap_v2::crc16};
use std::time::Instant;

fn wrapped(opcode: u8, value: u8) -> Vec<u8> {
    let mut bytes = vec![0x78, 0x9C, 0, 11, 2, 1, 2, opcode, value];
    bytes.extend_from_slice(&crc16(&bytes).to_be_bytes());
    bytes
}

#[test]
fn split_frames_ack_corruption_and_reconnect_cannot_confirm_wrong_transaction() {
    let good = wrapped(0x53, 1);
    for split in 1..good.len() {
        let mut epoch = SessionEpoch::default();
        let origin = epoch.token();
        let now = Instant::now();
        let mut receiver = NotificationReceiver::default();
        let mut corrupt = good.clone();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(receiver.push(&corrupt, now).is_empty());
        let ack = receiver.push(&[0xAA, 0x54, 1], now);
        assert_eq!(ack.len(), 1);
        assert!(Bp1ProAnc::decode_frame(&Frame::decode_notify(&ack[0]).unwrap(), None).is_err());
        assert!(receiver.push(&good[..split], now).is_empty());
        let frames = receiver.push(&good[split..], now);
        assert_eq!(frames.len(), 1);
        let frame = Frame::decode_notify(&frames[0]).unwrap();
        let observation = StateObservation { session: origin, opcode: frame.cmd,
            event: Bp1ProAnc::decode_frame(&frame, None).unwrap() };
        assert!(ExpectedState::Bass(1).matches(origin, &observation));
        assert!(!ExpectedState::Bass(0).matches(origin, &observation));
        epoch.invalidate();
        assert!(!ExpectedState::Bass(1).matches(epoch.token(), &observation));
    }
}
