//! Session identity is independent of Bluetooth addresses: reconnecting the
//! same device must invalidate work from its previous connection.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionToken(u64);

pub struct SessionEpoch {
    generation: u64,
    changes: tokio::sync::watch::Sender<u64>,
}

impl Default for SessionEpoch {
    fn default() -> Self {
        let (changes, _) = tokio::sync::watch::channel(0);
        Self {
            generation: 0,
            changes,
        }
    }
}

pub struct SessionLease {
    token: SessionToken,
    changes: tokio::sync::watch::Receiver<u64>,
}

impl SessionLease {
    pub async fn cancelled(&mut self) {
        loop {
            if *self.changes.borrow_and_update() != self.token.0 {
                return;
            }
            if self.changes.changed().await.is_err() {
                return;
            }
        }
    }
}

impl SessionEpoch {
    pub fn token(&self) -> SessionToken {
        SessionToken(self.generation)
    }

    pub fn invalidate(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("session epoch exhausted");
        self.changes.send_replace(self.generation);
    }

    pub fn accepts(&self, token: SessionToken) -> bool {
        self.token() == token
    }

    pub fn lease(&self, token: SessionToken) -> SessionLease {
        SessionLease {
            token,
            changes: self.changes.subscribe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_wakes_idle_task_and_late_subscriber() {
        let mut epoch = SessionEpoch::default();
        let token = epoch.token();
        let mut lease = epoch.lease(token);
        epoch.invalidate();
        tokio::time::timeout(std::time::Duration::from_millis(100), lease.cancelled())
            .await
            .unwrap();
        let mut late = epoch.lease(token);
        tokio::time::timeout(std::time::Duration::from_millis(100), late.cancelled())
            .await
            .unwrap();
    }

    #[test]
    fn reconnect_to_same_device_rejects_old_work() {
        let mut epoch = SessionEpoch::default();
        let first = epoch.token();
        assert!(epoch.accepts(first));
        epoch.invalidate();
        let second = epoch.token();
        assert!(!epoch.accepts(first));
        assert!(epoch.accepts(second));
        epoch.invalidate();
        assert!(!epoch.accepts(second));
    }
}
