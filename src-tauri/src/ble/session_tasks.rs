//! Own background work for exactly one BLE session generation.

#[derive(Default)]
pub(super) struct SessionTasks {
    session_id: Option<u64>,
    notification: Option<tokio::task::JoinHandle<()>>,
    battery_poller: Option<tokio::task::JoinHandle<()>>,
}

impl SessionTasks {
    pub(super) fn reset_for_session(&mut self, session_id: u64) {
        self.abort_all();
        self.session_id = Some(session_id);
    }

    pub(super) fn register_notification(
        &mut self,
        session_id: u64,
        task: tokio::task::JoinHandle<()>,
    ) -> bool {
        Self::register(&mut self.notification, self.session_id, session_id, task)
    }

    pub(super) fn register_battery_poller(
        &mut self,
        session_id: u64,
        task: tokio::task::JoinHandle<()>,
    ) -> bool {
        Self::register(&mut self.battery_poller, self.session_id, session_id, task)
    }

    fn register(
        slot: &mut Option<tokio::task::JoinHandle<()>>,
        current_session_id: Option<u64>,
        session_id: u64,
        task: tokio::task::JoinHandle<()>,
    ) -> bool {
        if current_session_id != Some(session_id) {
            task.abort();
            return false;
        }
        if let Some(previous) = slot.replace(task) {
            previous.abort();
        }
        true
    }

    fn abort_all(&mut self) {
        if let Some(task) = self.notification.take() {
            task.abort();
        }
        if let Some(task) = self.battery_poller.take() {
            task.abort();
        }
    }
}
