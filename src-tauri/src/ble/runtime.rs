//! Central owner for mutable BLE runtime state.

use super::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) struct BleInner {
    pub(super) adapter: Option<Adapter>,
    pub(super) central_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) notification_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) peripherals: HashMap<String, Peripheral>,
    pub(super) connected_id: Option<String>,
    pub(super) scanning: bool,
    pub(super) scan_generation: u64,
    pub(super) scan_revision: u64,
    pub(super) session: crate::device::session::SessionEpoch,
    pub(super) snapshot: crate::device::snapshot::DeviceSnapshot,
    pub(super) devices: HashMap<String, BleDevice>,
    /// Live battery merged from 0x02 + 0x27 notifies.
    pub(super) battery: BatteryState,
    pub(super) last_anc: Option<AncMode>,
    /// Best-effort record that a find-start write was accepted by the OS.
    pub(super) find_requested: bool,
    /// True when using mock scan/devices (no real GATT).
    pub(super) mock: bool,
    /// Live link diagnostics for UI.
    pub(super) has_write_uuid: bool,
    pub(super) has_notify_uuid: bool,
    pub(super) link_revision: u64,
    pub(super) handshake_ok: bool,
    pub(super) notify_count: u64,
    pub(super) tx_count: u64,
    pub(super) last_notify_ms: Option<u64>,
    pub(super) last_tx_ms: Option<u64>,
    pub(super) last_rx_hex: Option<String>,
    pub(super) last_tx_hex: Option<String>,
    pub(super) write_char: Option<String>,
    pub(super) notify_char: Option<String>,
}

impl BleInner {
    pub(super) fn new() -> Self {
        Self {
            adapter: None,
            central_task: None,
            notification_task: None,
            peripherals: HashMap::new(),
            connected_id: None,
            scanning: false,
            scan_generation: 0,
            scan_revision: 0,
            session: Default::default(),
            snapshot: crate::device::snapshot::DeviceSnapshot::new(0),
            devices: HashMap::new(),
            battery: BatteryState::default(),
            last_anc: None,
            find_requested: false,
            mock: false,
            has_write_uuid: false,
            has_notify_uuid: false,
            link_revision: 0,
            handshake_ok: false,
            notify_count: 0,
            tx_count: 0,
            last_notify_ms: None,
            last_tx_ms: None,
            last_rx_hex: None,
            last_tx_hex: None,
            write_char: None,
            notify_char: None,
        }
    }

    pub(super) fn reset_link(&mut self) {
        self.session.invalidate();
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
        self.touch_link();
        self.snapshot = crate::device::snapshot::DeviceSnapshot::new(self.session.token().id());
        self.has_write_uuid = false;
        self.has_notify_uuid = false;
        self.handshake_ok = false;
        self.notify_count = 0;
        self.tx_count = 0;
        self.last_notify_ms = None;
        self.last_tx_ms = None;
        self.last_rx_hex = None;
        self.last_tx_hex = None;
        self.write_char = None;
        self.notify_char = None;
        self.find_requested = false;
    }

    pub(super) fn touch_link(&mut self) {
        self.link_revision = self.link_revision.saturating_add(1);
    }
}

pub(super) static BLE: Lazy<Arc<Mutex<BleInner>>> =
    Lazy::new(|| Arc::new(Mutex::new(BleInner::new())));
