//! Central owner for mutable BLE runtime state.

use super::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) struct SessionRuntime {
    epoch: crate::device::session::SessionEpoch,
    tasks: super::session_tasks::SessionTasks,
    executor: Arc<crate::device::executor::CommandExecutor>,
    peripheral: Option<Peripheral>,
}

impl Default for SessionRuntime {
    fn default() -> Self {
        Self {
            epoch: Default::default(),
            tasks: Default::default(),
            executor: Arc::new(Default::default()),
            peripheral: None,
        }
    }
}

impl SessionRuntime {
    pub(super) fn token(&self) -> crate::device::session::SessionToken {
        self.epoch.token()
    }

    pub(super) fn accepts(&self, token: crate::device::session::SessionToken) -> bool {
        self.epoch.accepts(token)
    }

    pub(super) fn lease(
        &self,
        token: crate::device::session::SessionToken,
    ) -> crate::device::session::SessionLease {
        self.epoch.lease(token)
    }

    pub(super) fn command_executor(&self) -> Arc<crate::device::executor::CommandExecutor> {
        self.executor.clone()
    }

    pub(super) fn set_peripheral(
        &mut self,
        token: crate::device::session::SessionToken,
        peripheral: Peripheral,
    ) -> bool {
        if !self.epoch.accepts(token) || self.peripheral.is_some() {
            return false;
        }
        self.peripheral = Some(peripheral);
        true
    }

    pub(super) fn take_peripheral(&mut self) -> Option<Peripheral> {
        self.peripheral.take()
    }

    pub(super) fn peripheral(&self) -> Option<Peripheral> {
        self.peripheral.clone()
    }

    pub(super) fn register_notification(
        &mut self,
        token: crate::device::session::SessionToken,
        task: tokio::task::JoinHandle<()>,
    ) -> bool {
        self.tasks.register_notification(token.id(), task)
    }

    pub(super) fn register_battery_poller(
        &mut self,
        token: crate::device::session::SessionToken,
        task: tokio::task::JoinHandle<()>,
    ) -> bool {
        self.tasks.register_battery_poller(token.id(), task)
    }

    fn reset(&mut self) -> Vec<tokio::task::JoinHandle<()>> {
        self.epoch.invalidate();
        self.peripheral = None;
        let tasks = self.tasks.reset_for_session(self.epoch.token().id());
        self.executor = Arc::new(Default::default());
        tasks
    }
}

pub(super) struct BleInner {
    pub(super) peripherals: HashMap<String, Peripheral>,
    pub(super) connected_id: Option<String>,
    pub(super) scanning: bool,
    pub(super) scan_generation: u64,
    pub(super) scan_revision: u64,
    pub(super) session: SessionRuntime,
    pub(super) snapshot: crate::device::snapshot::DeviceSnapshot,
    pub(super) devices: HashMap<String, BleDevice>,
    /// Live battery merged from 0x02 + 0x27 notifies.
    pub(super) battery: BatteryState,
    pub(super) last_anc: Option<AncMode>,
    pub(super) diagnostics: super::diagnostics::LinkDiagnostics,
    /// Best-effort record that a find-start write was accepted by the OS.
    pub(super) find_requested: bool,
    /// True when using mock scan/devices (no real GATT).
    pub(super) mock: bool,
    /// Live link diagnostics for UI.
    pub(super) has_write_uuid: bool,
    pub(super) has_notify_uuid: bool,
    pub(super) link_revision: u64,
    pub(super) handshake_ok: bool,
    pub(super) write_char: Option<String>,
    pub(super) notify_char: Option<String>,
}

impl BleInner {
    pub(super) fn new() -> Self {
        Self {
            peripherals: HashMap::new(),
            connected_id: None,
            scanning: false,
            scan_generation: 0,
            scan_revision: 0,
            session: SessionRuntime::default(),
            snapshot: crate::device::snapshot::DeviceSnapshot::new(0),
            devices: HashMap::new(),
            battery: BatteryState::default(),
            last_anc: None,
            diagnostics: Default::default(),
            find_requested: false,
            mock: false,
            has_write_uuid: false,
            has_notify_uuid: false,
            link_revision: 0,
            handshake_ok: false,
            write_char: None,
            notify_char: None,
        }
    }

    pub(super) fn reset_link(&mut self) -> Vec<tokio::task::JoinHandle<()>> {
        let tasks = self.session.reset();
        self.touch_link();
        self.snapshot = crate::device::snapshot::DeviceSnapshot::new(self.session.token().id());
        self.has_write_uuid = false;
        self.has_notify_uuid = false;
        self.handshake_ok = false;
        self.diagnostics = Default::default();
        self.write_char = None;
        self.notify_char = None;
        self.find_requested = false;
        tasks
    }

    pub(super) fn touch_link(&mut self) {
        self.link_revision = self.link_revision.saturating_add(1);
    }
}

pub(super) async fn join_session_tasks(tasks: Vec<tokio::task::JoinHandle<()>>) {
    for task in tasks {
        let _ = task.await;
    }
}

#[cfg(test)]
mod tests {
    use super::{join_session_tasks, BleInner};

    #[tokio::test]
    async fn resetting_session_aborts_and_returns_its_owned_tasks_for_joining() {
        let mut state = BleInner::new();
        let _ = state.reset_link();
        let session_id = state.session.token().id();
        let notification = tokio::spawn(std::future::pending::<()>());
        let notification_abort = notification.abort_handle();
        let session_token = state.session.token();
        assert_eq!(session_token.id(), session_id);
        assert!(state
            .session
            .register_notification(session_token, notification));
        let poller = tokio::spawn(std::future::pending::<()>());
        let poller_abort = poller.abort_handle();
        assert!(state.session.register_battery_poller(session_token, poller));

        let tasks = state.reset_link();
        assert_eq!(tasks.len(), 2);
        join_session_tasks(tasks).await;

        assert!(notification_abort.is_finished());
        assert!(poller_abort.is_finished());
    }

    #[tokio::test]
    async fn old_session_task_is_aborted_if_registration_loses_the_reset_race() {
        let mut state = BleInner::new();
        let _ = state.reset_link();
        let old_session_token = state.session.token();
        let _ = state.reset_link();

        let stale_task = tokio::spawn(std::future::pending::<()>());
        let abort_handle = stale_task.abort_handle();

        assert!(!state
            .session
            .register_notification(old_session_token, stale_task));
        tokio::task::yield_now().await;
        assert!(abort_handle.is_finished());
    }

    #[tokio::test]
    async fn duplicate_role_registration_keeps_the_owned_task_and_aborts_the_duplicate() {
        let mut state = BleInner::new();
        let _ = state.reset_link();
        let session_token = state.session.token();
        let owned = tokio::spawn(std::future::pending::<()>());
        let owned_abort = owned.abort_handle();
        assert!(state.session.register_notification(session_token, owned));

        let duplicate = tokio::spawn(std::future::pending::<()>());
        let duplicate_abort = duplicate.abort_handle();
        assert!(!state
            .session
            .register_notification(session_token, duplicate));
        tokio::task::yield_now().await;
        assert!(!owned_abort.is_finished());
        assert!(duplicate_abort.is_finished());

        let tasks = state.reset_link();
        assert_eq!(tasks.len(), 1);
        join_session_tasks(tasks).await;
        assert!(owned_abort.is_finished());
    }

    #[test]
    fn resetting_session_replaces_its_command_executor() {
        let mut state = BleInner::new();
        let old_executor = state.session.command_executor();

        let _ = state.reset_link();

        assert!(!std::sync::Arc::ptr_eq(
            &old_executor,
            &state.session.command_executor()
        ));
    }
}

pub(super) static BLE: Lazy<Arc<Mutex<BleInner>>> =
    Lazy::new(|| Arc::new(Mutex::new(BleInner::new())));
