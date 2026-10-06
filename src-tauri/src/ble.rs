//! Bluetooth LE manager for multi-model earbuds (B4S).

pub mod commands;
pub mod connection;
mod contracts;
#[path = "ble/discovery.rs"]
mod discovery;
mod mock;
mod runtime;
pub mod scanning;
mod transport;
pub use contracts::{
    BleDevice, ConnectingState, ConnectionState, LinkHealth, LinkLevel, ScanStatus,
};
use runtime::BLE;
use transport::{
    with_connected_peripheral, write_and_observe, write_and_readback, write_bytes, write_command,
    write_raw,
};
#[path = "ble/handshake.rs"]
mod handshake;
pub use mock::{mock_connect, start_mock_scan};

use crate::protocol::{self, AncMode, BatteryState, DeviceEvent, EqPreset, ListeningCommand};
use btleplug::api::{
    Central, CentralEvent, CentralState, CharPropFlags, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use once_cell::sync::{Lazy, OnceCell};
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
        contract_version: 2,
        generation: state.scan_generation,
        revision: state.scan_revision,
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
        assert_eq!(encoded["contractVersion"], 2);
        assert_eq!(encoded["sessionId"], 0);
        assert_eq!(encoded["revision"], 0);
        assert_eq!(encoded["level"], "offline");
        assert_eq!(
            serde_json::from_value::<LinkLevel>(encoded["level"].clone()).unwrap(),
            LinkLevel::Offline
        );
        assert!(serde_json::from_value::<LinkLevel>(serde_json::json!("unknown")).is_err());
    }

    #[test]
    fn scan_contract_carries_generation_and_revision() {
        let encoded = serde_json::to_value(ScanStatus {
            contract_version: 2,
            generation: 7,
            revision: 12,
            scanning: true,
            devices: Vec::new(),
            error: None,
        })
        .unwrap();
        assert_eq!(encoded["contractVersion"], 2);
        assert_eq!(encoded["generation"], 7);
        assert_eq!(encoded["revision"], 12);
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
            contract_version: 2,
            session_id: state.session.token().id(),
            revision: state.link_revision,
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
        contract_version: 2,
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
