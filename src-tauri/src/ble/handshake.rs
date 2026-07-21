//! Deterministic first-connect state machine extracted from the APK flow.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Connecting,
    DiscoveringServices,
    EnablingNotifications,
    AwaitingInitState,
    Ready,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitState {
    Initial,
    AlreadyConfigured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindAction {
    RelieveBind,
    PreserveExisting,
}

pub struct Handshake {
    phase: Phase,
    timeout_ms: u64,
    elapsed_ms: u64,
    bind_action: Option<BindAction>,
}

impl Handshake {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            phase: Phase::Connecting,
            timeout_ms,
            elapsed_ms: 0,
            bind_action: None,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn on_connected(&mut self) {
        if self.phase == Phase::Connecting {
            self.phase = Phase::DiscoveringServices;
        }
    }

    pub fn on_services_discovered(&mut self) {
        if self.phase == Phase::DiscoveringServices {
            self.phase = Phase::EnablingNotifications;
        }
    }

    pub fn on_notifications_enabled(&mut self) {
        if self.phase == Phase::EnablingNotifications {
            self.phase = Phase::AwaitingInitState;
        }
    }

    pub fn init_payload(&self) -> &'static [u8] {
        b"#InitState:"
    }

    pub fn on_init_state(&mut self, state: InitState) {
        if self.phase == Phase::AwaitingInitState {
            self.bind_action = Some(match state {
                InitState::Initial => BindAction::RelieveBind,
                InitState::AlreadyConfigured => BindAction::PreserveExisting,
            });
            self.phase = Phase::Ready;
        }
    }

    pub fn on_elapsed(&mut self, elapsed_ms: u64) -> Phase {
        self.elapsed_ms = self.elapsed_ms.saturating_add(elapsed_ms);
        if self.phase != Phase::Ready && self.elapsed_ms >= self.timeout_ms {
            self.phase = Phase::TimedOut;
        }
        self.phase
    }

    pub fn bind_action(&self) -> Option<BindAction> {
        self.bind_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_through_notify_and_init_state_phases() {
        let mut flow = Handshake::new(30_000);
        assert_eq!(flow.phase(), Phase::Connecting);
        flow.on_connected();
        flow.on_services_discovered();
        flow.on_notifications_enabled();
        assert_eq!(flow.init_payload(), b"#InitState:");
        assert_eq!(flow.phase(), Phase::AwaitingInitState);
        flow.on_init_state(InitState::AlreadyConfigured);
        assert_eq!(flow.phase(), Phase::Ready);
        assert_eq!(flow.bind_action(), Some(BindAction::PreserveExisting));
    }

    #[test]
    fn initial_state_requests_relieve_bind() {
        let mut flow = Handshake::new(30_000);
        flow.on_connected();
        flow.on_services_discovered();
        flow.on_notifications_enabled();
        flow.on_init_state(InitState::Initial);
        assert_eq!(flow.phase(), Phase::Ready);
        assert_eq!(flow.bind_action(), Some(BindAction::RelieveBind));
    }

    #[test]
    fn timeout_is_recoverable_until_ready() {
        let mut flow = Handshake::new(30_000);
        flow.on_connected();
        flow.on_services_discovered();
        flow.on_notifications_enabled();
        assert_eq!(flow.on_elapsed(30_001), Phase::TimedOut);
        assert_eq!(flow.bind_action(), None);
    }
}
