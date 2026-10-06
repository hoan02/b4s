//! Versioned BLE data contracts shared by the runtime and Tauri boundary.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub address: String,
    pub rssi: i16,
    pub is_baseus: bool,
    pub connected: bool,
    pub headphone_candidate: bool,
    /// Matched catalog model id (e.g. bass-bp1-pro)
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub device_profile: crate::protocol::DeviceProfile,
    /// verified | experimental | scanOnly
    pub support: Option<String>,
    /// UI hint when a platform lists the same product twice (BLE control vs audio)
    pub hint: Option<String>,
    pub image_url: Option<String>,
    pub image_provenance: String,
    pub color_variants: Vec<String>,
    pub serial: Option<String>,
    pub advertised_services: Vec<String>,
}

/// How healthy the control link is — UI uses this to show Demo / Waiting / Live / Dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkLevel {
    Live,
    Waiting,
    Dead,
    Demo,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkHealth {
    pub contract_version: u16,
    pub session_id: u64,
    pub revision: u64,
    /// Frontend session thinks we are connected
    pub connected: bool,
    /// True when using mock scan/devices (no real GATT)
    pub mock: bool,
    /// btleplug reports peripheral still connected
    pub peripheral_connected: bool,
    /// Exact Baseus write UUID found
    pub has_write_uuid: bool,
    /// Exact Baseus notify UUID found
    pub has_notify_uuid: bool,
    /// Handshake BA 05 00 write succeeded
    pub handshake_ok: bool,
    /// Number of GATT notifications received since connect
    pub notify_count: u64,
    /// Number of successful TX writes since connect
    pub tx_count: u64,
    pub last_notify_ms: Option<u64>,
    pub last_tx_ms: Option<u64>,
    pub last_rx_hex: Option<String>,
    pub last_tx_hex: Option<String>,
    pub write_char: Option<String>,
    pub notify_char: Option<String>,
    /// live | waiting | dead | demo | offline
    pub level: LinkLevel,
    /// Human-readable summary for the UI
    pub message: String,
}

impl Default for LinkHealth {
    fn default() -> Self {
        Self {
            contract_version: 2,
            session_id: 0,
            revision: 0,
            connected: false,
            mock: false,
            peripheral_connected: false,
            has_write_uuid: false,
            has_notify_uuid: false,
            handshake_ok: false,
            notify_count: 0,
            tx_count: 0,
            last_notify_ms: None,
            last_tx_ms: None,
            last_rx_hex: None,
            last_tx_hex: None,
            write_char: None,
            notify_char: None,
            level: LinkLevel::Offline,
            message: "Not connected".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionState {
    pub contract_version: u16,
    pub connected: bool,
    pub device: Option<BleDevice>,
    pub error: Option<String>,
    pub link: LinkHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectingState {
    pub contract_version: u16,
    pub device_id: String,
    pub session_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub contract_version: u16,
    pub generation: u64,
    pub revision: u64,
    pub scanning: bool,
    pub devices: Vec<BleDevice>,
    pub error: Option<String>,
}
