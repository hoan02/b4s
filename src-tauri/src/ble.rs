//! Bluetooth LE manager for multi-model earbuds (B4S).

pub mod commands;
pub mod connection;
#[path = "ble/discovery.rs"]
mod discovery;
mod runtime;
pub mod scanning;
mod transport;
use runtime::BLE;
use transport::{
    with_connected_peripheral, write_and_readback, write_bytes, write_command, write_raw,
};
#[path = "ble/handshake.rs"]
mod handshake;

use crate::device::{DeviceIdentity, DeviceRegistry};
use crate::protocol::{self, AncMode, BatteryState, DeviceEvent, EqPreset, ListeningCommand};
use btleplug::api::{
    Central, CentralEvent, CentralState, CharPropFlags, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Set once from lib.rs setup for event emit helpers.
static OBSERVATIONS: Lazy<
    tokio::sync::broadcast::Sender<crate::device::confirmation::StateObservation>,
> = Lazy::new(|| tokio::sync::broadcast::channel(64).0);

static COMMAND_EXECUTOR: Lazy<crate::device::executor::CommandExecutor> =
    Lazy::new(Default::default);

static CONNECT_ATTEMPT: Mutex<()> = Mutex::const_new(());

static APP: OnceCell<AppHandle> = OnceCell::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP.set(app);
}

fn app_handle() -> Option<&'static AppHandle> {
    APP.get()
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

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
    pub device_profile: protocol::DeviceProfile,
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
            contract_version: 1,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub contract_version: u16,
    pub scanning: bool,
    pub devices: Vec<BleDevice>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn compute_link_level(h: &LinkHealth) -> (LinkLevel, String) {
    if !h.connected {
        return (LinkLevel::Offline, "Not connected".into());
    }
    if h.mock {
        return (
            LinkLevel::Demo,
            "Demo mode — UI only, not talking to real earbuds".into(),
        );
    }
    if !h.peripheral_connected {
        return (
            LinkLevel::Dead,
            "BLE link dropped — disconnect and scan again".into(),
        );
    }
    if !h.has_write_uuid || !h.has_notify_uuid {
        return (
            LinkLevel::Dead,
            "GATT control service missing — not a BP1 protocol device, or Windows pairing incomplete".into(),
        );
    }
    if h.notify_count > 0 {
        return (
            LinkLevel::Live,
            format!(
                "Live link · {} notifies · {} writes",
                h.notify_count, h.tx_count
            ),
        );
    }
    if h.handshake_ok {
        return (
            LinkLevel::Waiting,
            "The profile handshake was written; waiting for a framed device response.".into(),
        );
    }
    (
        LinkLevel::Dead,
        "The selected device did not complete its reviewed control handshake.".into(),
    )
}

fn normalize_addr(a: &str) -> String {
    a.to_uppercase().replace('-', ":")
}

fn model_fields(name: &str) -> (bool, Option<String>, Option<String>, Option<String>) {
    let resolved = DeviceRegistry::resolve(DeviceIdentity {
        address: String::new(),
        name: name.into(),
        advertised_service: None,
        manufacturer_data: Vec::new(),
    });
    if let Some(m) = resolved.model {
        let support = match m.support {
            protocol::SupportLevel::Verified => "verified",
            protocol::SupportLevel::Experimental => "experimental",
            protocol::SupportLevel::ScanOnly => "scanOnly",
        };
        (true, Some(m.id), Some(m.display_name), Some(support.into()))
    } else {
        (false, None, None, None)
    }
}

fn id_to_string(id: &PeripheralId) -> String {
    format!("{:?}", id)
}

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

pub async fn init_adapter() -> Result<(), String> {
    let mut state = BLE.lock().await;
    if state.adapter.is_some() {
        return Ok(());
    }
    let manager = Manager::new()
        .await
        .map_err(|e| format!("BLE manager: {e}"))?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|e| format!("List adapters: {e}"))?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or_else(|| "No Bluetooth adapter found".to_string())?;
    log::info!("BLE adapter ready");
    state.adapter = Some(adapter);
    Ok(())
}

pub async fn is_adapter_available() -> bool {
    // Adapter present and radio usable across Windows, macOS, and Linux.
    if init_adapter().await.is_err() {
        return false;
    }
    let state = BLE.lock().await;
    let Some(adapter) = state.adapter.clone() else {
        return false;
    };
    drop(state);
    // Read the platform radio state. An adapter can exist while Bluetooth is
    // powered off, so probing scan alone is not a reliable power-state check.
    match adapter.adapter_state().await {
        Ok(CentralState::PoweredOn) => return true,
        Ok(CentralState::PoweredOff) => return false,
        _ => {}
    }

    // Probe: start + stop scan for unknown backend states.
    match adapter.start_scan(ScanFilter::default()).await {
        Ok(()) => {
            let _ = adapter.stop_scan().await;
            true
        }
        Err(e) => {
            log::warn!("Bluetooth not usable (off/disabled?): {e}");
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Scan
// ---------------------------------------------------------------------------

async fn ensure_session(token: crate::device::session::SessionToken) -> Result<(), String> {
    if BLE.lock().await.session.accepts(token) {
        Ok(())
    } else {
        Err("Connection attempt was cancelled".into())
    }
}

fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Write helpers
// ---------------------------------------------------------------------------

fn model_presentation(model_id: Option<&str>) -> (Option<String>, String, Vec<String>) {
    let Some(id) = model_id else {
        return (None, "fallback".into(), Vec::new());
    };
    protocol::catalog_json()
        .into_iter()
        .find(|model| model.id == id)
        .map(|model| {
            (
                model.image_url,
                model.image_provenance,
                model.color_variants,
            )
        })
        .unwrap_or((None, "fallback".into(), Vec::new()))
}

pub async fn get_scan_status() -> ScanStatus {
    let state = BLE.lock().await;
    let mut devices = visible_scan_devices(state.devices.values().cloned().collect());
    devices.sort_by(|a, b| {
        (b.is_baseus, advertises_reviewed_service(b), b.rssi).cmp(&(
            a.is_baseus,
            advertises_reviewed_service(a),
            a.rssi,
        ))
    });
    ScanStatus {
        contract_version: 1,
        scanning: state.scanning,
        devices,
        error: None,
    }
}

fn advertises_reviewed_service(device: &BleDevice) -> bool {
    let Some(service_uuid) = device
        .device_profile
        .connection
        .as_ref()
        .and_then(|connection| connection.service_uuid.as_deref())
    else {
        return false;
    };
    device
        .advertised_services
        .iter()
        .any(|uuid| uuid.eq_ignore_ascii_case(service_uuid))
}

fn visible_scan_devices(all: Vec<BleDevice>) -> Vec<BleDevice> {
    all
}

#[cfg(test)]
mod scan_tests {
    use super::*;

    fn device(id: &str, rssi: i16, services: &[&str]) -> BleDevice {
        BleDevice {
            id: id.into(),
            name: "Baseus Bass BP1 Pro".into(),
            address: id.into(),
            rssi,
            is_baseus: true,
            connected: false,
            headphone_candidate: true,
            model_id: Some("bass-bp1-pro".into()),
            model_name: Some("Baseus Bass BP1 Pro".into()),
            device_profile: protocol::profile_for(
                Some("bass-bp1-pro"),
                Some("Baseus Bass BP1 Pro"),
                None,
            ),
            support: Some("verified".into()),
            hint: None,
            image_url: None,
            image_provenance: "fallback".into(),
            color_variants: Vec::new(),
            serial: None,
            advertised_services: services.iter().map(|s| (*s).into()).collect(),
        }
    }

    #[test]
    fn scan_list_keeps_same_name_entries_independently_selectable() {
        let devices = visible_scan_devices(vec![
            device("audio", -45, &[]),
            device(
                "control",
                -70,
                &[protocol::advertisement::BASEUS_SERVICE_UUID],
            ),
        ]);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "audio");
        assert_eq!(devices[1].id, "control");
    }

    #[test]
    fn link_contract_is_versioned_and_uses_a_closed_level_enum() {
        let encoded = serde_json::to_value(LinkHealth::default()).unwrap();
        assert_eq!(encoded["contractVersion"], 1);
        assert_eq!(encoded["level"], "offline");
        assert_eq!(
            serde_json::from_value::<LinkLevel>(encoded["level"].clone()).unwrap(),
            LinkLevel::Offline
        );
        assert!(serde_json::from_value::<LinkLevel>(serde_json::json!("unknown")).is_err());
    }
}

pub async fn get_connection_state() -> ConnectionState {
    let (connected, device, mock, mut link_partial, peripheral) = {
        let state = BLE.lock().await;
        let connected = state.connected_id.is_some();
        let device = state
            .connected_id
            .as_ref()
            .and_then(|id| state.devices.get(id).cloned());
        let peripheral = state
            .connected_id
            .as_ref()
            .and_then(|id| state.peripherals.get(id).cloned());
        let partial = LinkHealth {
            contract_version: 1,
            connected,
            mock: state.mock,
            peripheral_connected: false, // filled below
            has_write_uuid: state.has_write_uuid,
            has_notify_uuid: state.has_notify_uuid,
            handshake_ok: state.handshake_ok,
            notify_count: state.notify_count,
            tx_count: state.tx_count,
            last_notify_ms: state.last_notify_ms,
            last_tx_ms: state.last_tx_ms,
            last_rx_hex: state.last_rx_hex.clone(),
            last_tx_hex: state.last_tx_hex.clone(),
            write_char: state.write_char.clone(),
            notify_char: state.notify_char.clone(),
            level: LinkLevel::Offline,
            message: String::new(),
        };
        (connected, device, state.mock, partial, peripheral)
    };

    let peripheral_connected = if mock {
        connected
    } else if let Some(p) = peripheral {
        p.is_connected().await.unwrap_or(false)
    } else {
        false
    };
    link_partial.peripheral_connected = peripheral_connected;
    let (level, message) = compute_link_level(&link_partial);
    link_partial.level = level;
    link_partial.message = message;

    ConnectionState {
        contract_version: 1,
        connected,
        device,
        error: None,
        link: link_partial,
    }
}

pub async fn get_link_health() -> LinkHealth {
    get_connection_state().await.link
}

async fn emit_scan_status(app: &AppHandle) {
    let _ = app.emit("ble://scan-status", &get_scan_status().await);
}

async fn emit_connection_state(app: &AppHandle) {
    let state = get_connection_state().await;
    #[cfg(desktop)]
    crate::desktop::update_tray_status(app, &state);
    let _ = app.emit("ble://connection", &state);
}

// ---------------------------------------------------------------------------
// Mock mode
// ---------------------------------------------------------------------------

pub async fn start_mock_scan(app: AppHandle) -> Result<(), String> {
    let mut state = BLE.lock().await;
    state.scanning = true;
    state.scan_generation = state.scan_generation.wrapping_add(1);
    let scan_generation = state.scan_generation;
    state.mock = true;
    state.devices.clear();
    drop(state);
    emit_scan_status(&app).await;

    let mock_names: [(&str, &str, i16); 6] = [
        ("mock-bp1", "Bass BP1 Pro", -42),
        ("mock-ma10", "Baseus Bowie MA10", -51),
        ("mock-ma10s", "Bowie MA10s", -55),
        ("mock-m2s", "Bowie M2s Pro", -60),
        ("mock-e3", "Bowie E3", -63),
        ("mock-inspire", "Inspire XP1", -48),
    ];
    let mocks: Vec<BleDevice> = mock_names
        .iter()
        .map(|(id, name, rssi)| {
            let (is_baseus, model_id, model_name, support) = model_fields(name);
            let (image_url, image_provenance, color_variants) =
                model_presentation(model_id.as_deref());
            let device_profile =
                protocol::profile_for(model_id.as_deref(), model_name.as_deref(), None);
            BleDevice {
                id: (*id).into(),
                name: (*name).into(),
                address: format!("AA:BB:CC:DD:EE:{:02X}", rssi.unsigned_abs() % 200),
                rssi: *rssi,
                is_baseus,
                connected: false,
                headphone_candidate: true,
                model_id,
                model_name,
                device_profile,
                support,
                hint: None,
                image_url,
                image_provenance,
                color_variants,
                serial: None,
                advertised_services: Vec::new(),
            }
        })
        .collect();

    for (i, dev) in mocks.into_iter().enumerate() {
        let app2 = app.clone();
        let d = dev.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(350 * (i as u64 + 1))).await;
            {
                let mut s = BLE.lock().await;
                if !s.scanning || s.scan_generation != scan_generation {
                    return;
                }
                s.devices.insert(d.id.clone(), d.clone());
            }
            emit_scan_status(&app2).await;
        });
    }

    let app3 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        let mut s = BLE.lock().await;
        if s.scan_generation != scan_generation {
            return;
        }
        s.scanning = false;
        drop(s);
        emit_scan_status(&app3).await;
    });
    Ok(())
}

pub async fn mock_connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _attempt = CONNECT_ATTEMPT
        .try_lock()
        .map_err(|_| "Another connection attempt is active")?;
    let _ = scanning::stop_scan(app.clone()).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut state = BLE.lock().await;
    state.mock = true;
    state.reset_link();
    // Demo link: fake health so UI clearly shows DEMO, not Live
    state.has_write_uuid = false;
    state.has_notify_uuid = false;
    state.handshake_ok = false;
    state.notify_count = 0;
    state.tx_count = 0;
    let mut device = state
        .devices
        .get_mut(&device_id)
        .ok_or("Mock device not found")?
        .clone();
    device.connected = true;
    state.connected_id = Some(device_id);
    if let Some(d) = state.devices.get_mut(&device.id) {
        d.connected = true;
    }
    // Seed mock battery (fake values — NOT from hardware)
    state.battery = BatteryState {
        left: 87,
        right: 92,
        case: 64,
        left_charging: false,
        right_charging: false,
        case_charging: true,
    };
    let bat = state.battery.clone();
    state.snapshot.device_id = Some(device.id.clone());
    state.snapshot.model_id = device.model_id.clone();
    state.snapshot.mock = true;
    state
        .snapshot
        .observe(2, &DeviceEvent::Battery(bat.clone()), now_ms());
    state
        .snapshot
        .observe(0x27, &DeviceEvent::Battery(bat.clone()), now_ms());
    state
        .snapshot
        .observe(0x34, &DeviceEvent::Anc(AncMode::Anc), now_ms());
    state
        .snapshot
        .observe(0x42, &DeviceEvent::Eq(EqPreset::Balanced), now_ms());
    state
        .snapshot
        .observe(0x23, &DeviceEvent::GameMode(false), now_ms());
    drop(state);

    emit_connection_state(&app).await;
    let snapshot = BLE.lock().await.snapshot.clone();
    let _ = app.emit("device://snapshot", &snapshot);
    Ok(device)
}
