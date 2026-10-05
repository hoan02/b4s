//! Session identity is independent of Bluetooth addresses: reconnecting the
//! same device must invalidate work from its previous connection.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionToken(u64);

#[derive(Default)]
pub struct SessionEpoch(u64);

impl SessionEpoch {
    pub fn token(&self) -> SessionToken {
        SessionToken(self.0)
    }

    pub fn invalidate(&mut self) {
        self.0 = self.0.checked_add(1).expect("session epoch exhausted");
    }

    pub fn accepts(&self, token: SessionToken) -> bool {
        self.token() == token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
