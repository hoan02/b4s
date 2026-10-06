//! Connection-local reassembly using only the framing declared by the model profile.

use super::wrap_v2::unwrap_notify;
use crate::catalog::WireFraming;
use std::time::{Duration, Instant};

const MAX_FRAME: usize = 4096;
const ASSEMBLY_TIMEOUT: Duration = Duration::from_secs(2);

pub struct NotificationReceiver {
    framing: WireFraming,
    pending: Vec<u8>,
    started: Option<Instant>,
}

impl NotificationReceiver {
    pub fn new(framing: WireFraming) -> Self {
        Self {
            framing,
            pending: Vec::new(),
            started: None,
        }
    }
}

impl NotificationReceiver {
    pub fn push(&mut self, chunk: &[u8], now: Instant) -> Vec<Vec<u8>> {
        match self.framing {
            WireFraming::BareAaBa => {
                return if chunk.first() == Some(&0xAA) {
                    vec![chunk.to_vec()]
                } else {
                    Vec::new()
                };
            }
            WireFraming::Unresolved => return Vec::new(),
            WireFraming::Headphone789c => {}
        }
        if self
            .started
            .is_some_and(|start| now.saturating_duration_since(start) >= ASSEMBLY_TIMEOUT)
        {
            self.reset();
        }
        // Do not search arbitrary input for embedded magic or battery patterns.
        if self.pending.is_empty() && chunk.first() != Some(&0x78) {
            return Vec::new();
        }
        if self.pending.len().saturating_add(chunk.len()) > MAX_FRAME {
            self.reset();
            return Vec::new();
        }
        self.started.get_or_insert(now);
        self.pending.extend_from_slice(chunk);
        let mut frames = Vec::new();
        loop {
            if self.pending.len() < 2 {
                break;
            }
            if self.pending[..2] != [0x78, 0x9C] {
                self.reset();
                break;
            }
            if self.pending.len() < 4 {
                break;
            }
            let total = u16::from_be_bytes([self.pending[2], self.pending[3]]) as usize;
            if !(8..=MAX_FRAME).contains(&total) {
                self.reset();
                break;
            }
            if self.pending.len() < total {
                break;
            }
            frames.extend(unwrap_notify(&self.pending[..total]));
            self.pending.drain(..total);
            if self.pending.is_empty() {
                self.reset();
                break;
            }
            self.started = Some(now);
        }
        frames
    }

    pub fn reset(&mut self) {
        self.pending.clear();
        self.started = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::wrap_v2::crc16;

    fn battery() -> Vec<u8> {
        let mut packet = vec![0x78, 0x9C, 0, 14, 2, 1, 5, 2, 0, 0, 4, 1];
        packet.extend_from_slice(&crc16(&packet).to_be_bytes());
        packet
    }

    #[test]
    fn every_split_and_batched_frame_preserves_low_battery() {
        let packet = battery();
        for split in 1..packet.len() {
            let mut receiver = NotificationReceiver::new(WireFraming::Headphone789c);
            let now = Instant::now();
            assert!(receiver.push(&packet[..split], now).is_empty());
            assert_eq!(
                receiver.push(&packet[split..], now),
                vec![vec![0xAA, 2, 0, 0, 4, 1]]
            );
        }
        let mut receiver = NotificationReceiver::new(WireFraming::Headphone789c);
        let batched = [packet.clone(), packet].concat();
        assert_eq!(receiver.push(&batched, Instant::now()).len(), 2);
    }

    #[test]
    fn timeout_disconnect_and_oversize_do_not_leak_partial_state() {
        let packet = battery();
        let now = Instant::now();
        let mut receiver = NotificationReceiver::new(WireFraming::Headphone789c);
        receiver.push(&packet[..4], now);
        assert!(receiver
            .push(&packet[4..], now + ASSEMBLY_TIMEOUT)
            .is_empty());
        receiver.push(&packet[..4], now);
        receiver.reset();
        assert!(receiver.push(&packet[4..], now).is_empty());
        assert!(receiver.push(&[0x78; MAX_FRAME + 1], now).is_empty());
        assert_eq!(receiver.push(&packet, now).len(), 1);
    }

    #[test]
    fn corrupt_frame_does_not_poison_following_frame() {
        let packet = battery();
        let mut bad = packet.clone();
        bad[13] ^= 1;
        let mut receiver = NotificationReceiver::new(WireFraming::Headphone789c);
        assert_eq!(
            receiver.push(&[bad, packet].concat(), Instant::now()).len(),
            1
        );
    }

    #[test]
    fn profile_framing_rejects_other_wire_formats() {
        let now = Instant::now();
        let mut bare = NotificationReceiver::new(WireFraming::BareAaBa);
        assert_eq!(
            bare.push(&[0xAA, 0x02, 0x01], now),
            vec![vec![0xAA, 0x02, 0x01]]
        );
        assert!(bare.push(&battery(), now).is_empty());

        let mut wrapped = NotificationReceiver::new(WireFraming::Headphone789c);
        assert!(wrapped.push(&[0xAA, 0x02, 0x01], now).is_empty());

        let mut unresolved = NotificationReceiver::new(WireFraming::Unresolved);
        assert!(unresolved.push(&[0xAA, 0x02, 0x01], now).is_empty());
        assert!(unresolved.push(&battery(), now).is_empty());
    }
}
