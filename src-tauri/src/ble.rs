//! Bluetooth LE manager for Baseus earbuds.
//!
//! Uses btleplug + Baseus protocol (BP1 Pro ANC packet table).

use crate::capture::{self, Direction as CapDir, GattChar};
use crate::protocol::{
    self, AncMode, BatteryState, Bp1ProAnc, DeviceEvent, EqPreset,
};
use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Set once from lib.rs setup so write_raw can log to Capture Studio.
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
    /// Matched catalog model id (e.g. bass-bp1-pro)
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    /// verified | experimental | scanOnly
    pub support: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionState {
    pub connected: bool,
    pub device: Option<BleDevice>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub scanning: bool,
    pub devices: Vec<BleDevice>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

struct BleInner {
    adapter: Option<Adapter>,
    peripherals: HashMap<String, Peripheral>,
    connected_id: Option<String>,
    scanning: bool,
    devices: HashMap<String, BleDevice>,
    /// Live battery merged from 0x02 + 0x27 notifies
    battery: BatteryState,
    last_anc: Option<AncMode>,
    /// true when using mock (no real GATT)
    mock: bool,
}

impl BleInner {
    fn new() -> Self {
        Self {
            adapter: None,
            peripherals: HashMap::new(),
            connected_id: None,
            scanning: false,
            devices: HashMap::new(),
            battery: BatteryState::default(),
            last_anc: None,
            mock: false,
        }
    }
}

static BLE: Lazy<Arc<Mutex<BleInner>>> = Lazy::new(|| Arc::new(Mutex::new(BleInner::new())));

fn is_likely_baseus(name: &str) -> bool {
    protocol::looks_like_baseus(name)
}

fn model_fields(name: &str) -> (bool, Option<String>, Option<String>, Option<String>) {
    if let Some(m) = protocol::identify_model(name) {
        let support = match m.support {
            protocol::SupportLevel::Verified => "verified",
            protocol::SupportLevel::Experimental => "experimental",
            protocol::SupportLevel::ScanOnly => "scanOnly",
        };
        (
            true,
            Some(m.id),
            Some(m.display_name),
            Some(support.into()),
        )
    } else if protocol::looks_like_baseus(name) {
        (true, None, None, Some("scanOnly".into()))
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
    init_adapter().await.is_ok()
}

// ---------------------------------------------------------------------------
// Scan
// ---------------------------------------------------------------------------

pub async fn start_scan(app: AppHandle) -> Result<(), String> {
    init_adapter().await?;
    let mut state = BLE.lock().await;
    if state.scanning {
        return Ok(());
    }
    let adapter = state
        .adapter
        .as_ref()
        .ok_or("Adapter not initialized")?
        .clone();
    let connected = state.connected_id.clone();
    state.devices.retain(|id, _| Some(id.clone()) == connected);
    state.peripherals.retain(|id, _| Some(id.clone()) == connected);
    state.scanning = true;
    state.mock = false;
    drop(state);

    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("start_scan: {e}"))?;
    emit_scan_status(&app).await;

    let app_c = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = listen_central_events(app_c, adapter).await {
            log::error!("central events: {e}");
        }
    });

    let app_t = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let _ = stop_scan(app_t).await;
    });
    Ok(())
}

async fn listen_central_events(app: AppHandle, adapter: Adapter) -> Result<(), String> {
    let mut events = adapter
        .events()
        .await
        .map_err(|e| format!("events: {e}"))?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                if let Ok(p) = adapter.peripheral(&id).await {
                    process_peripheral(&app, p, &id).await;
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                let id_str = id_to_string(&id);
                let mut state = BLE.lock().await;
                if state.connected_id.as_ref() == Some(&id_str) {
                    state.connected_id = None;
                    state.battery = BatteryState::default();
                }
                if let Some(d) = state.devices.get_mut(&id_str) {
                    d.connected = false;
                }
                drop(state);
                emit_connection_state(&app).await;
                let _ = app.emit("ble://disconnected", &id_str);
            }
            _ => {}
        }
    }
    Ok(())
}

async fn process_peripheral(app: &AppHandle, peripheral: Peripheral, id: &PeripheralId) {
    let props = match peripheral.properties().await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let name = props.local_name.unwrap_or_default();
    if name.is_empty() {
        return;
    }
    let id_str = id_to_string(id);
    let (is_baseus, model_id, model_name, support) = model_fields(&name);
    let device = BleDevice {
        id: id_str.clone(),
        name: name.clone(),
        address: props.address.to_string(),
        rssi: props.rssi.unwrap_or(-100),
        is_baseus,
        connected: false,
        model_id,
        model_name,
        support,
    };
    {
        let mut state = BLE.lock().await;
        state.devices.insert(id_str.clone(), device.clone());
        state.peripherals.insert(id_str, peripheral);
    }
    let _ = app.emit("ble://device", &device);
    emit_scan_status(app).await;
}

pub async fn stop_scan(app: AppHandle) -> Result<(), String> {
    let mut state = BLE.lock().await;
    if !state.scanning {
        return Ok(());
    }
    if let Some(a) = &state.adapter {
        let _ = a.stop_scan().await;
    }
    state.scanning = false;
    drop(state);
    emit_scan_status(&app).await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Connect / subscribe to Baseus GATT
// ---------------------------------------------------------------------------

pub async fn connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _ = stop_scan(app.clone()).await;

    let mut state = BLE.lock().await;
    let peripheral = state
        .peripherals
        .get(&device_id)
        .ok_or_else(|| format!("Device {device_id} not found — scan again"))?
        .clone();
    state.mock = false;
    drop(state);

    log::info!("Connecting to {device_id}…");
    let _ = app.emit("ble://connecting", &device_id);

    peripheral
        .connect()
        .await
        .map_err(|e| format!("Connect failed: {e}"))?;

    peripheral
        .discover_services()
        .await
        .map_err(|e| format!("Service discovery: {e}"))?;

    // Build GATT map for Capture Studio
    let gatt_map = build_gatt_map(&peripheral);
    capture::set_gatt_map(app.clone(), gatt_map).await;

    // Subscribe to Baseus notify characteristic
    subscribe_notifications(app.clone(), peripheral.clone()).await?;

    // Query current EQ after connect
    let _ = write_raw(&peripheral, &Bp1ProAnc::cmd_query_eq()).await;

    let mut state = BLE.lock().await;
    state.connected_id = Some(device_id.clone());
    let device = if let Some(d) = state.devices.get_mut(&device_id) {
        d.connected = true;
        d.clone()
    } else {
        BleDevice {
            id: device_id,
            name: "Baseus".into(),
            address: String::new(),
            rssi: 0,
            is_baseus: true,
            connected: true,
            model_id: None,
            model_name: None,
            support: Some("experimental".into()),
        }
    };
    drop(state);

    capture::set_device_info(Some(device.name.clone()), Some(device.address.clone())).await;

    emit_connection_state(&app).await;
    let _ = app.emit("ble://connected", &device);
    Ok(device)
}

fn build_gatt_map(peripheral: &Peripheral) -> Vec<GattChar> {
    let mut map = Vec::new();
    for service in peripheral.services() {
        for ch in &service.characteristics {
            let mut props = Vec::new();
            let p = ch.properties;
            if p.contains(CharPropFlags::READ) {
                props.push("READ".into());
            }
            if p.contains(CharPropFlags::WRITE) {
                props.push("WRITE".into());
            }
            if p.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                props.push("WRITE_NR".into());
            }
            if p.contains(CharPropFlags::NOTIFY) {
                props.push("NOTIFY".into());
            }
            if p.contains(CharPropFlags::INDICATE) {
                props.push("INDICATE".into());
            }
            map.push(GattChar {
                uuid: ch.uuid.to_string(),
                properties: props,
                service_uuid: service.uuid.to_string(),
            });
        }
    }
    map
}

async fn subscribe_notifications(app: AppHandle, peripheral: Peripheral) -> Result<(), String> {
    let notify_uuid = protocol::uuids::notify();
    let chars = peripheral.characteristics();
    let notify_char = chars
        .iter()
        .find(|c| c.uuid == notify_uuid)
        .ok_or_else(|| {
            // Fallback: any notify characteristic
            log::warn!("Baseus notify UUID not found — trying first notify char");
            "Baseus notify characteristic not found".to_string()
        });

    let ch = match notify_char {
        Ok(c) => c.clone(),
        Err(_) => {
            // Try any char with Notify
            chars
                .iter()
                .find(|c| c.properties.contains(CharPropFlags::NOTIFY))
                .cloned()
                .ok_or("No NOTIFY characteristic on device")?
        }
    };

    peripheral
        .subscribe(&ch)
        .await
        .map_err(|e| format!("Subscribe failed: {e}"))?;

    log::info!("Subscribed to notifications on {}", ch.uuid);

    // Notification listener
    let mut stream = peripheral
        .notifications()
        .await
        .map_err(|e| format!("notifications stream: {e}"))?;

    tauri::async_runtime::spawn(async move {
        while let Some(n) = stream.next().await {
            log::debug!("Notify {} : {:02X?}", n.uuid, n.value);
            handle_notification(&app, &n.value, Some(n.uuid.to_string())).await;
        }
        log::info!("Notification stream ended");
    });

    Ok(())
}

async fn handle_notification(app: &AppHandle, data: &[u8], char_uuid: Option<String>) {
    let last_anc = {
        let state = BLE.lock().await;
        state.last_anc
    };

    let decoded = match protocol::decode_notification(data, last_anc) {
        Ok(event) => {
            log::info!("DeviceEvent: {:?}", event);
            let hint = format!("{event:?}");
            apply_event(app, event).await;
            Some(hint)
        }
        Err(e) => {
            log::debug!("Decode skip: {e}  raw={:02X?}", data);
            let _ = app.emit(
                "ble://raw",
                &serde_json::json!({ "hex": hex_encode(data) }),
            );
            capture::hint_decode(data)
        }
    };

    // Capture Studio
    capture::record(
        app,
        CapDir::Rx,
        data,
        char_uuid,
        decoded.or_else(|| capture::hint_decode(data)),
    )
    .await;
}

async fn apply_event(app: &AppHandle, event: DeviceEvent) {
    match &event {
        DeviceEvent::Battery(partial) => {
            let mut state = BLE.lock().await;
            // 0x02 frame: has L/R (case fields stay 0 in partial)
            // 0x27 frame: has case (L/R stay 0 in partial)
            // 0% is valid (bud seated in case) — always apply non-default side
            let from_buds = partial.left != 0 || partial.right != 0
                || (!partial.case_charging && partial.case == 0 && partial.left == 0 && partial.right == 0);
            // Heuristic: if case field set or case_charging, treat as case frame
            let from_case = partial.case != 0 || partial.case_charging
                || (partial.left == 0 && partial.right == 0);

            if from_buds && !from_case {
                state.battery.left = partial.left;
                state.battery.right = partial.right;
                state.battery.left_charging = partial.left_charging;
                state.battery.right_charging = partial.right_charging;
            } else if from_case && partial.left == 0 && partial.right == 0 {
                state.battery.case = partial.case;
                state.battery.case_charging = partial.case_charging;
            } else {
                // Full update
                state.battery.left = partial.left;
                state.battery.right = partial.right;
                state.battery.case = if partial.case != 0 { partial.case } else { state.battery.case };
                state.battery.left_charging = partial.left_charging;
                state.battery.right_charging = partial.right_charging;
                if partial.case_charging {
                    state.battery.case_charging = true;
                }
            }
            let bat = state.battery.clone();
            drop(state);
            let _ = app.emit("device://battery", &bat);
        }
        DeviceEvent::Anc(mode) => {
            let mut state = BLE.lock().await;
            state.last_anc = Some(*mode);
            drop(state);
            let _ = app.emit("device://anc", mode);
        }
        DeviceEvent::Eq(preset) => {
            let _ = app.emit("device://eq", preset);
        }
        DeviceEvent::GameMode(on) => {
            let _ = app.emit("device://game", on);
        }
        DeviceEvent::Unknown { cmd, payload } => {
            let _ = app.emit(
                "ble://raw",
                &serde_json::json!({ "cmd": cmd, "hex": hex_encode(payload) }),
            );
        }
    }
    let _ = app.emit("device://event", &event);
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Write helpers
// ---------------------------------------------------------------------------

async fn write_raw(peripheral: &Peripheral, data: &[u8]) -> Result<(), String> {
    let write_uuid = protocol::uuids::write();
    let chars = peripheral.characteristics();
    let ch = chars
        .iter()
        .find(|c| c.uuid == write_uuid)
        .or_else(|| {
            chars.iter().find(|c| {
                c.properties.contains(CharPropFlags::WRITE)
                    || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
            })
        })
        .ok_or("No WRITE characteristic")?;

    let write_type = if ch.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
        WriteType::WithoutResponse
    } else {
        WriteType::WithResponse
    };

    log::info!("Write {:02X?} → {}", data, ch.uuid);
    peripheral
        .write(ch, data, write_type)
        .await
        .map_err(|e| format!("Write failed: {e}"))?;

    // Capture Studio — log TX
    if let Some(app) = app_handle() {
        capture::record(
            app,
            CapDir::Tx,
            data,
            Some(ch.uuid.to_string()),
            capture::hint_decode(data),
        )
        .await;
    }
    Ok(())
}

/// Write arbitrary hex bytes (Capture Studio raw write).
pub async fn write_hex(hex: &str) -> Result<(), String> {
    let data = parse_hex(hex)?;
    if data.is_empty() {
        return Err("Empty payload".into());
    }
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_raw(&p, &d).await })
    })
    .await
    .or_else(|e| {
        if e == "MOCK" {
            // Log to capture even in mock
            if let Some(app) = app_handle() {
                let app = app.clone();
                let d = data.clone();
                tauri::async_runtime::spawn(async move {
                    capture::record(
                        &app,
                        CapDir::Tx,
                        &d,
                        None,
                        capture::hint_decode(&d),
                    )
                    .await;
                });
            }
            Ok(())
        } else {
            Err(e)
        }
    })
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let clean: String = s
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    if clean.len() % 2 != 0 {
        return Err("Hex string must have even length".into());
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean[i..i + 2], 16)
                .map_err(|e| format!("Bad hex at {i}: {e}"))
        })
        .collect()
}

async fn with_connected_peripheral<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(Peripheral) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
{
    let state = BLE.lock().await;
    if state.mock {
        return Err("MOCK".into());
    }
    let id = state
        .connected_id
        .as_ref()
        .ok_or("Not connected")?
        .clone();
    let p = state
        .peripherals
        .get(&id)
        .ok_or("Peripheral gone")?
        .clone();
    drop(state);
    f(p).await
}

/// Log TX (+ optional mock RX ack) when capture is running in mock mode.
async fn mock_tx_rx(tx: &[u8], rx_ack: Option<&[u8]>) {
    if let Some(app) = app_handle() {
        capture::record(app, CapDir::Tx, tx, None, capture::hint_decode(tx)).await;
        if let Some(rx) = rx_ack {
            tokio::time::sleep(Duration::from_millis(40)).await;
            capture::record(app, CapDir::Rx, rx, None, capture::hint_decode(rx)).await;
        }
    }
}

pub async fn send_anc(mode: AncMode, strength: u8) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_anc(mode, strength);
    {
        let mut state = BLE.lock().await;
        state.last_anc = Some(mode);
        if state.mock {
            drop(state);
            let ack = [0xAA, 0x34, mode.to_byte()];
            mock_tx_rx(&data, Some(&ack)).await;
            if let Some(app) = app_handle() {
                let _ = app.emit("device://anc", &mode);
            }
            return Ok(());
        }
    }
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_raw(&p, &d).await })
    })
    .await
}

pub async fn send_eq(preset: EqPreset) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_eq(preset);
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        let ack = [0xAA, 0x43, preset.to_byte()];
        mock_tx_rx(&data, Some(&ack)).await;
        if let Some(app) = app_handle() {
            let _ = app.emit("device://eq", &preset);
        }
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_raw(&p, &d).await })
    })
    .await
}

pub async fn send_game_mode(on: bool) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_game_mode(on);
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        let ack = [0xAA, 0x23, if on { 0x01 } else { 0x00 }];
        mock_tx_rx(&data, Some(&ack)).await;
        if let Some(app) = app_handle() {
            let _ = app.emit("device://game", &on);
        }
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_raw(&p, &d).await })
    })
    .await
}

pub async fn send_find_buds() -> Result<(), String> {
    let data = Bp1ProAnc::cmd_find_buds();
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        mock_tx_rx(&data, None).await;
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_raw(&p, &d).await })
    })
    .await
}

pub async fn get_battery_state() -> BatteryState {
    BLE.lock().await.battery.clone()
}

// ---------------------------------------------------------------------------
// Disconnect
// ---------------------------------------------------------------------------

pub async fn disconnect(app: AppHandle) -> Result<(), String> {
    let mut state = BLE.lock().await;
    let id = match &state.connected_id {
        Some(id) => id.clone(),
        None => return Ok(()),
    };
    if let Some(p) = state.peripherals.get(&id) {
        let p = p.clone();
        drop(state);
        let _ = p.disconnect().await;
    } else {
        drop(state);
    }
    let mut state = BLE.lock().await;
    state.connected_id = None;
    state.battery = BatteryState::default();
    state.last_anc = None;
    if let Some(d) = state.devices.get_mut(&id) {
        d.connected = false;
    }
    drop(state);
    emit_connection_state(&app).await;
    let _ = app.emit("ble://disconnected", &id);
    Ok(())
}

// ---------------------------------------------------------------------------
// Queries / emit
// ---------------------------------------------------------------------------

pub async fn get_scan_status() -> ScanStatus {
    let state = BLE.lock().await;
    let mut devices: Vec<_> = state.devices.values().cloned().collect();
    devices.sort_by(|a, b| match (a.is_baseus, b.is_baseus) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.rssi.cmp(&a.rssi),
    });
    ScanStatus {
        scanning: state.scanning,
        devices,
        error: None,
    }
}

pub async fn get_connection_state() -> ConnectionState {
    let state = BLE.lock().await;
    let device = state
        .connected_id
        .as_ref()
        .and_then(|id| state.devices.get(id).cloned());
    ConnectionState {
        connected: state.connected_id.is_some(),
        device,
        error: None,
    }
}

async fn emit_scan_status(app: &AppHandle) {
    let _ = app.emit("ble://scan-status", &get_scan_status().await);
}

async fn emit_connection_state(app: &AppHandle) {
    let _ = app.emit("ble://connection", &get_connection_state().await);
}

// ---------------------------------------------------------------------------
// Mock mode
// ---------------------------------------------------------------------------

pub async fn start_mock_scan(app: AppHandle) -> Result<(), String> {
    let mut state = BLE.lock().await;
    state.scanning = true;
    state.mock = true;
    state.devices.clear();
    drop(state);
    emit_scan_status(&app).await;

    let mock_names = [
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
            BleDevice {
                id: (*id).into(),
                name: (*name).into(),
                address: format!("AA:BB:CC:DD:EE:{:02X}", rssi.unsigned_abs() % 200),
                rssi: *rssi,
                is_baseus,
                connected: false,
                model_id,
                model_name,
                support,
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
                if !s.scanning {
                    return;
                }
                s.devices.insert(d.id.clone(), d.clone());
            }
            let _ = app2.emit("ble://device", &d);
            emit_scan_status(&app2).await;
        });
    }

    let app3 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        let mut s = BLE.lock().await;
        s.scanning = false;
        drop(s);
        emit_scan_status(&app3).await;
    });
    Ok(())
}

pub async fn mock_connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _ = stop_scan(app.clone()).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut state = BLE.lock().await;
    state.mock = true;
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
    // Seed mock battery
    state.battery = BatteryState {
        left: 87,
        right: 92,
        case: 64,
        left_charging: false,
        right_charging: false,
        case_charging: true,
    };
    let bat = state.battery.clone();
    drop(state);

    emit_connection_state(&app).await;
    let _ = app.emit("ble://connected", &device);
    let _ = app.emit("device://battery", &bat);

    // Simulate periodic battery notify
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let _ = app2.emit("device://anc", &AncMode::Anc);
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = app2.emit("device://eq", &EqPreset::Balanced);
    });

    Ok(device)
}
