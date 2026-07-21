//! Bluetooth LE manager for multi-model earbuds (B4S).

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

/// Set once from lib.rs setup for event emit helpers.
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
    /// UI hint when Windows lists the same product twice (BLE control vs audio)
    pub hint: Option<String>,
}

/// How healthy the control link is — UI uses this to show Demo / Waiting / Live / Dead.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkHealth {
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
    pub level: String,
    /// Human-readable summary for the UI
    pub message: String,
}

impl Default for LinkHealth {
    fn default() -> Self {
        Self {
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
            level: "offline".into(),
            message: "Not connected".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionState {
    pub connected: bool,
    pub device: Option<BleDevice>,
    pub error: Option<String>,
    pub link: LinkHealth,
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
    /// Live link diagnostics for UI
    has_write_uuid: bool,
    has_notify_uuid: bool,
    handshake_ok: bool,
    notify_count: u64,
    tx_count: u64,
    last_notify_ms: Option<u64>,
    last_tx_ms: Option<u64>,
    last_rx_hex: Option<String>,
    last_tx_hex: Option<String>,
    write_char: Option<String>,
    notify_char: Option<String>,
    /// Use 789C+CRC wrap (BP1 Ultra / N0 models from official app)
    use_v2_wrap: bool,
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
            use_v2_wrap: false,
        }
    }

    fn reset_link(&mut self) {
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
        self.use_v2_wrap = false;
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn compute_link_level(h: &LinkHealth) -> (String, String) {
    if !h.connected {
        return ("offline".into(), "Not connected".into());
    }
    if h.mock {
        return (
            "demo".into(),
            "Demo mode — UI only, not talking to real earbuds".into(),
        );
    }
    if !h.peripheral_connected {
        return (
            "dead".into(),
            "BLE link dropped — disconnect and scan again".into(),
        );
    }
    if !h.has_write_uuid || !h.has_notify_uuid {
        return (
            "dead".into(),
            "GATT control service missing — not a BP1 protocol device, or Windows pairing incomplete".into(),
        );
    }
    if h.notify_count > 0 {
        return (
            "live".into(),
            format!(
                "Live link · {} notifies · {} writes",
                h.notify_count, h.tx_count
            ),
        );
    }
    if h.handshake_ok {
        return (
            "waiting".into(),
            "GATT write OK, waiting for notify/battery. Open case lid or wear buds. BP1 Ultra uses 789C framing — if still silent, official app may be using Classic BT (SPP) for this model.".into(),
        );
    }
    (
        "dead".into(),
        "BLE connected but control write/handshake failed. Try the other scan entry with the same name, or forget+re-pair buds.".into(),
    )
}

fn normalize_addr(a: &str) -> String {
    a.to_uppercase().replace('-', ":")
}

static BLE: Lazy<Arc<Mutex<BleInner>>> = Lazy::new(|| Arc::new(Mutex::new(BleInner::new())));

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
    // Adapter present AND radio usable (start_scan fails when BT is off on Windows)
    if init_adapter().await.is_err() {
        return false;
    }
    let state = BLE.lock().await;
    let Some(adapter) = state.adapter.clone() else {
        return false;
    };
    drop(state);
    // Probe: start + stop scan. If Bluetooth is powered off this errors.
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
                    state.reset_link();
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
    let address = props.address.to_string();
    let rssi = props.rssi.unwrap_or(-100);
    let (is_baseus, model_id, model_name, support) = model_fields(&name);

    let mut device = BleDevice {
        id: id_str.clone(),
        name: name.clone(),
        address: address.clone(),
        rssi,
        is_baseus,
        connected: false,
        model_id,
        model_name,
        support,
        hint: None,
    };

    {
        let mut state = BLE.lock().await;
        // Prefer stronger RSSI if same Windows id already seen
        if let Some(old) = state.devices.get(&id_str) {
            if rssi < old.rssi {
                // keep stronger
                return;
            }
        }
        // Same MAC already listed under another id → keep stronger, drop weaker
        let addr_key = normalize_addr(&address);
        if !addr_key.is_empty() && addr_key != "00:00:00:00:00:00" {
            let mut drop_ids = Vec::new();
            for (oid, od) in state.devices.iter() {
                if oid != &id_str && normalize_addr(&od.address) == addr_key {
                    if rssi >= od.rssi {
                        drop_ids.push(oid.clone());
                    } else {
                        return; // existing is stronger
                    }
                }
            }
            for oid in drop_ids {
                state.devices.remove(&oid);
                state.peripherals.remove(&oid);
            }
        }

        // Same display name twice (common on Windows: BLE control GATT + audio LE)
        // Different MAC/id → keep both and label clearly so user knows which to try.
        let same_name: Vec<(String, i16, String)> = state
            .devices
            .iter()
            .filter(|(oid, d)| *oid != &id_str && d.name.eq_ignore_ascii_case(&name))
            .map(|(oid, d)| (oid.clone(), d.rssi, d.address.clone()))
            .collect();
        if !same_name.is_empty() {
            let stronger = same_name.iter().all(|(_, r, _)| rssi >= *r);
            device.hint = Some(if stronger {
                "Cùng tên #1 (RSSI mạnh hơn) — ưu tiên thử entry này (thường là BLE control)."
                    .into()
            } else {
                "Cùng tên #2 — nếu connect xong không ANC/EQ/pin được thì thử entry kia."
                    .into()
            });
            for (oid, other_rssi, _) in &same_name {
                if let Some(d) = state.devices.get_mut(oid) {
                    let this_stronger = *other_rssi >= rssi;
                    d.hint = Some(if this_stronger {
                        "Cùng tên #1 (RSSI mạnh hơn) — ưu tiên thử entry này (thường là BLE control)."
                            .into()
                    } else {
                        "Cùng tên #2 — nếu connect xong không ANC/EQ/pin được thì thử entry kia."
                            .into()
                    });
                }
            }
        }

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

/// Fresh peripheral handle from adapter (avoids Windows "object has been closed").
async fn resolve_peripheral(device_id: &str) -> Result<Peripheral, String> {
    init_adapter().await?;
    let (adapter, known_addr, known_name) = {
        let state = BLE.lock().await;
        if let Some(p) = state.peripherals.get(device_id) {
            return Ok(p.clone());
        }
        let adapter = state
            .adapter
            .as_ref()
            .ok_or("Adapter not initialized")?
            .clone();
        let dev = state.devices.get(device_id);
        (
            adapter,
            dev.map(|d| d.address.clone()),
            dev.map(|d| d.name.clone()),
        )
    };

    let peris = adapter
        .peripherals()
        .await
        .map_err(|e| format!("list peripherals: {e}"))?;
    for p in peris {
        let id = id_to_string(&p.id());
        if id == device_id {
            let mut state = BLE.lock().await;
            state.peripherals.insert(id, p.clone());
            return Ok(p);
        }
        // Match by address / name after reconnect (id string can change on Windows)
        if let Ok(Some(props)) = p.properties().await {
            let addr = props.address.to_string();
            let name = props.local_name.unwrap_or_default();
            let addr_ok = known_addr
                .as_ref()
                .map(|a| !a.is_empty() && a.eq_ignore_ascii_case(&addr))
                .unwrap_or(false);
            let name_ok = known_name
                .as_ref()
                .map(|n| !n.is_empty() && n.eq_ignore_ascii_case(&name))
                .unwrap_or(false);
            if addr_ok || name_ok {
                let mut state = BLE.lock().await;
                // Re-key under original device_id for session continuity
                state.peripherals.insert(device_id.to_string(), p.clone());
                if let Some(d) = state.devices.get_mut(device_id) {
                    d.address = addr;
                    if !name.is_empty() {
                        d.name = name;
                    }
                }
                return Ok(p);
            }
        }
    }
    Err(format!(
        "Device not found after disconnect — tap Scan again, then Connect ({device_id})"
    ))
}

/// Other scan entries with the same BLE name (Windows dual audio/control).
fn sibling_ids(state: &BleInner, device_id: &str) -> Vec<String> {
    let name = match state.devices.get(device_id) {
        Some(d) if !d.name.is_empty() => d.name.clone(),
        _ => return vec![],
    };
    let mut sibs: Vec<(String, i16)> = state
        .devices
        .iter()
        .filter(|(id, d)| {
            *id != device_id && d.is_baseus && d.name.eq_ignore_ascii_case(&name)
        })
        .map(|(id, d)| (id.clone(), d.rssi))
        .collect();
    // Prefer stronger RSSI first
    sibs.sort_by(|a, b| b.1.cmp(&a.1));
    sibs.into_iter().map(|(id, _)| id).collect()
}

fn has_control_chars(peripheral: &Peripheral) -> (bool, bool) {
    let chars = peripheral.characteristics();
    let has_write = chars.iter().any(|c| {
        c.uuid == protocol::uuids::write()
            || c.uuid == protocol::uuids::ccsdk_write()
            || c.properties.contains(CharPropFlags::WRITE)
            || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
    });
    let has_notify = chars.iter().any(|c| {
        c.uuid == protocol::uuids::notify()
            || c.uuid == protocol::uuids::ccsdk_notify()
            || c.properties.contains(CharPropFlags::NOTIFY)
    });
    let has_baseus = chars.iter().any(|c| {
        c.uuid == protocol::uuids::write()
            || c.uuid == protocol::uuids::notify()
            || c.uuid == protocol::uuids::ccsdk_write()
            || c.uuid == protocol::uuids::ccsdk_notify()
    });
    // Prefer known Baseus UUIDs; generic write+notify alone is weak (audio LE)
    (has_write && has_notify, has_baseus)
}

pub async fn connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _ = stop_scan(app.clone()).await;

    // If already connected to something, disconnect cleanly first
    {
        let state = BLE.lock().await;
        if state.connected_id.is_some() {
            drop(state);
            let _ = disconnect(app.clone()).await;
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    // Build try list: user pick + same-name siblings (dual Windows entries)
    let mut try_ids = vec![device_id.clone()];
    {
        let state = BLE.lock().await;
        for sid in sibling_ids(&state, &device_id) {
            if !try_ids.contains(&sid) {
                try_ids.push(sid);
            }
        }
    }

    let mut last_err = String::from("Connect failed");
    for (i, id) in try_ids.iter().enumerate() {
        let mut state = BLE.lock().await;
        state.mock = false;
        state.reset_link();
        drop(state);

        log::info!(
            "Connect attempt {}/{} → {id}",
            i + 1,
            try_ids.len()
        );
        let _ = app.emit("ble://connecting", id);

        match connect_one(app.clone(), id.clone()).await {
            Ok(dev) => {
                if i > 0 {
                    log::info!(
                        "Connected via sibling entry (Windows dual list) — original was {device_id}"
                    );
                }
                return Ok(dev);
            }
            Err(e) => {
                log::warn!("Connect {id} failed: {e}");
                last_err = e;
                // Clean half-open before next sibling
                if let Ok(p) = resolve_peripheral(id).await {
                    let _ = p.disconnect().await;
                }
                {
                    let mut state = BLE.lock().await;
                    state.connected_id = None;
                    state.reset_link();
                    state.peripherals.remove(id);
                }
                tokio::time::sleep(Duration::from_millis(350)).await;
            }
        }
    }

    if try_ids.len() > 1 {
        Err(format!(
            "{last_err} — đã thử {} entry cùng tên. Forget buds trong Windows Bluetooth rồi pair lại khi mở nắp hộp.",
            try_ids.len()
        ))
    } else {
        Err(last_err)
    }
}

async fn connect_one(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let peripheral = resolve_peripheral(&device_id).await?;

    // Ensure not half-open
    if peripheral.is_connected().await.unwrap_or(false) {
        let _ = peripheral.disconnect().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    peripheral
        .connect()
        .await
        .map_err(|e| format!("Connect failed: {e}"))?;

    // Small delay then discover — Windows needs this after reconnect
    tokio::time::sleep(Duration::from_millis(250)).await;

    peripheral
        .discover_services()
        .await
        .map_err(|e| format!("Service discovery: {e}"))?;

    // Keep fresh handle in map
    {
        let mut state = BLE.lock().await;
        state.peripherals.insert(device_id.clone(), peripheral.clone());
    }

    let (has_generic, has_baseus) = has_control_chars(&peripheral);
    {
        let chars = peripheral.characteristics();
        let mut state = BLE.lock().await;
        state.has_write_uuid = has_generic || has_baseus;
        state.has_notify_uuid = has_generic || has_baseus;
        log::info!(
            "Control GATT: baseus={has_baseus} generic={has_generic} (chars={})",
            chars.len()
        );
        for c in &chars {
            log::info!("  char {} props={:?}", c.uuid, c.properties);
        }
    }

    if !has_baseus && !has_generic {
        let _ = peripheral.disconnect().await;
        return Err(
            "No control GATT on this entry (likely audio-only). Trying other same-name entry…"
                .into(),
        );
    }
    if !has_baseus {
        // Weak signal: may be wrong dual entry — still try, but prefer fail if subscribe dies
        log::warn!("No known Baseus write/notify UUID — may be wrong dual entry");
    }

    // Decide framing from catalog match
    let device_preview = {
        let state = BLE.lock().await;
        state.devices.get(&device_id).cloned()
    };
    let use_v2 = protocol::needs_v2_wrap(
        device_preview.as_ref().and_then(|d| d.model_id.as_deref()),
        device_preview.as_ref().and_then(|d| d.model_name.as_deref()),
        device_preview.as_ref().map(|d| d.name.as_str()),
    );
    {
        let mut state = BLE.lock().await;
        state.use_v2_wrap = use_v2;
    }
    log::info!("Protocol wrap_v2 (789C+CRC) = {use_v2}");

    // Subscribe to notify characteristic(s)
    subscribe_notifications(app.clone(), peripheral.clone()).await?;

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Handshake BA0500 — try both bare and wrapped always (Pro vs Ultra)
    let mut handshake_ok = false;
    let handshakes: Vec<Vec<u8>> = {
        let bare = vec![0xBA, 0x05, 0x00];
        let mut v = Vec::new();
        if let Some(w) = protocol::wrap_ba_command(&bare) {
            v.push(w);
        }
        v.push(bare);
        v.push(vec![0xBA, 0x05, 0x01]);
        if let Some(w) = protocol::wrap_ba_command(&[0xBA, 0x05, 0x01]) {
            v.push(w);
        }
        v
    };
    for pkt in handshakes {
        match write_raw(&peripheral, &pkt).await {
            Ok(()) => {
                log::info!("Handshake OK: {:02X?}", pkt);
                handshake_ok = true;
                // If wrapped handshake worked, stick to v2 for later writes
                if pkt.len() >= 2 && pkt[0] == 0x78 && pkt[1] == 0x9C {
                    let mut state = BLE.lock().await;
                    state.use_v2_wrap = true;
                    log::info!("Enabled wrap_v2 from successful 789C handshake");
                }
                break;
            }
            Err(e) => log::warn!("Handshake fail {:02X?}: {e}", pkt),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    {
        let mut state = BLE.lock().await;
        state.handshake_ok = handshake_ok;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Post-connect queries — battery first (BA02 bare + wrapped)
    let _ = send_battery_queries(&peripheral).await;
    tokio::time::sleep(Duration::from_millis(120)).await;
    let _ = write_command(&peripheral, protocol::Command::QueryEq).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let _ = write_bytes(&peripheral, &[0xBA, 0x23]).await;
    // Another battery nudge after EQ (some firmwares only answer once “awake”)
    tokio::time::sleep(Duration::from_millis(80)).await;
    let _ = send_battery_queries(&peripheral).await;

    let mut state = BLE.lock().await;
    state.connected_id = Some(device_id.clone());
    let mut device = if let Some(d) = state.devices.get_mut(&device_id) {
        d.connected = true;
        d.clone()
    } else {
        BleDevice {
            id: device_id.clone(),
            name: "Device".into(),
            address: String::new(),
            rssi: 0,
            is_baseus: true,
            connected: true,
            model_id: None,
            model_name: None,
            support: Some("experimental".into()),
            hint: None,
        }
    };
    if !has_baseus {
        device.hint = Some(
            "GATT generic — nếu ANC/pin không chạy, Disconnect rồi chọn entry cùng tên khác."
                .into(),
        );
    }
    drop(state);

    emit_connection_state(&app).await;
    let _ = app.emit("ble://connected", &device);

    // Poll battery + link health while connected
    let app_h = app.clone();
    let poll_id = device_id.clone();
    tauri::async_runtime::spawn(async move {
        for i in 0..40 {
            tokio::time::sleep(Duration::from_secs(if i < 5 { 2 } else { 10 })).await;
            let still = {
                let s = BLE.lock().await;
                s.connected_id.as_ref() == Some(&poll_id)
            };
            if !still {
                break;
            }
            if let Ok(p) = resolve_peripheral(&poll_id).await {
                let _ = send_battery_queries(&p).await;
            }
            emit_connection_state(&app_h).await;
        }
    });
    Ok(device)
}

async fn subscribe_notifications(app: AppHandle, peripheral: Peripheral) -> Result<(), String> {
    let notify_candidates = [
        protocol::uuids::notify(),
        protocol::uuids::ccsdk_notify(),
    ];
    let write_candidates = [
        protocol::uuids::write(),
        protocol::uuids::ccsdk_write(),
    ];
    let chars = peripheral.characteristics();

    // Pick ONE notify char (multi-subscribe on Windows often hits "object closed")
    let ch = notify_candidates
        .iter()
        .find_map(|u| chars.iter().find(|c| c.uuid == *u))
        .or_else(|| {
            chars
                .iter()
                .find(|c| c.properties.contains(CharPropFlags::NOTIFY))
        })
        .cloned()
        .ok_or_else(|| {
            "No NOTIFY characteristic. Forget buds in Windows Bluetooth, pair from this app while buds are in case-open/pairing mode.".to_string()
        })?;

    // Retry subscribe once after re-discover (stale handles after disconnect)
    let subscribe_result = peripheral.subscribe(&ch).await;
    let subscribe_result = match subscribe_result {
        Err(e) => {
            log::warn!("Subscribe first try failed ({e}), rediscover + retry");
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = peripheral.discover_services().await;
            let chars2 = peripheral.characteristics();
            let ch2 = chars2
                .iter()
                .find(|c| c.uuid == ch.uuid)
                .or_else(|| {
                    chars2
                        .iter()
                        .find(|c| c.properties.contains(CharPropFlags::NOTIFY))
                })
                .cloned()
                .ok_or_else(|| format!("Subscribe retry: char gone ({e})"))?;
            peripheral.subscribe(&ch2).await.map(|_| ch2)
        }
        Ok(()) => Ok(ch),
    };
    let ch = subscribe_result.map_err(|e| format!("Subscribe failed: {e}"))?;
    log::info!("Subscribed notify {}", ch.uuid);

    let has_write = chars.iter().any(|c| {
        write_candidates.contains(&c.uuid)
            || c.properties.contains(CharPropFlags::WRITE)
            || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
    });
    if !has_write {
        return Err(
            "No WRITE characteristic found. Control service missing — try the other scan entry with the same name (Windows often lists audio + BLE control as two devices)."
                .into(),
        );
    }

    {
        let mut state = BLE.lock().await;
        state.notify_char = Some(ch.uuid.to_string());
        state.has_notify_uuid = true;
        state.has_write_uuid = true;
    }

    let mut stream = peripheral
        .notifications()
        .await
        .map_err(|e| format!("notifications stream: {e}"))?;

    tauri::async_runtime::spawn(async move {
        while let Some(n) = stream.next().await {
            log::info!("Notify {} : {:02X?}", n.uuid, n.value);
            handle_notification(&app, &n.value, Some(n.uuid.to_string())).await;
        }
        log::info!("Notification stream ended");
    });

    Ok(())
}

async fn handle_notification(app: &AppHandle, data: &[u8], _char_uuid: Option<String>) {
    // Track RX for link health — strongest proof of a real device link
    {
        let mut state = BLE.lock().await;
        state.notify_count = state.notify_count.saturating_add(1);
        state.last_notify_ms = Some(now_ms());
        state.last_rx_hex = Some(hex_encode(data));
        // Auto-learn 789C framing if device speaks it
        if data.len() >= 2 && data[0] == 0x78 && data[1] == 0x9C && !state.use_v2_wrap {
            state.use_v2_wrap = true;
            log::info!("Enabled wrap_v2 from 789C notify");
        }
    }
    let _ = app.emit("ble://link", &get_connection_state().await.link);

    let last_anc = {
        let state = BLE.lock().await;
        state.last_anc
    };

    // Unwrap 789C multi-frames → one or more AA payloads; decode each
    // Official app: after unwrap, battery is AA02LL00RR01 (BleUtils.d)
    let frames = protocol::unwrap_notify(data);
    let mut any_decoded = false;
    for frame in &frames {
        match protocol::Frame::decode_notify(frame) {
            Ok(fr) => match protocol::Bp1ProAnc::decode_frame(&fr, last_anc) {
                Ok(event) => {
                    log::info!("DeviceEvent: {:?}", event);
                    apply_event(app, event).await;
                    any_decoded = true;
                }
                Err(e) => log::debug!("Decode skip: {e}  raw={:02X?}", frame),
            },
            Err(e) => log::debug!("Frame skip: {e}  raw={:02X?}", frame),
        }
    }

    // Salvage AA02 only when we still have no bud % — never on every mode ACK
    // (false AA02-looking bytes in ANC/EQ replies made % jump down)
    let need_battery = {
        let s = BLE.lock().await;
        s.battery.left == 0 && s.battery.right == 0
    };
    if need_battery {
        if let Some(bat) = salvage_battery_from_raw(data) {
            log::info!("Battery salvaged from raw notify: {:?}", bat);
            apply_event(app, DeviceEvent::Battery(bat)).await;
            any_decoded = true;
        }
    }

    if !any_decoded {
        let _ = app.emit(
            "ble://raw",
            &serde_json::json!({ "hex": hex_encode(data) }),
        );
    }
}

/// Official BleUtils.d only: AA 02 LL 00 RR 01 with both sides ≥ 5%.
fn salvage_battery_from_raw(data: &[u8]) -> Option<BatteryState> {
    for w in data.windows(6) {
        if w[0] == 0xAA && w[1] == 0x02 && w[3] == 0x00 && w[5] == 0x01 {
            let left = if w[2] > 100 { w[2] & 0x7F } else { w[2] }.min(100);
            let right = if w[4] > 100 { w[4] & 0x7F } else { w[4] }.min(100);
            if left >= 5 && right >= 5 {
                return Some(BatteryState {
                    left,
                    right,
                    case: 0,
                    left_charging: w[2] > 100,
                    right_charging: w[4] > 100,
                    case_charging: false,
                });
            }
        }
    }
    None
}

/// Reject junk battery readings that appear when switching ANC/EQ (mode ACKs
/// mis-parsed as AA02). Keep last good % sticky.
fn should_accept_buds(prev: &BatteryState, next: &BatteryState) -> bool {
    // Need at least one bud %
    if next.left == 0 && next.right == 0 {
        return false;
    }
    // Both sides must be plausible
    if next.left > 100 || next.right > 100 {
        return false;
    }
    // First reading — accept if either side looks real
    if prev.left == 0 && prev.right == 0 {
        return next.left >= 5 || next.right >= 5;
    }
    // Already have good values: reject sudden free-fall (e.g. 87% → 3%)
    // that is typical of mode-ACK noise, not real discharge
    let prev_avg = ((prev.left as u16 + prev.right as u16) / 2) as i16;
    let next_avg = {
        let l = if next.left > 0 { next.left } else { next.right };
        let r = if next.right > 0 { next.right } else { next.left };
        ((l as u16 + r as u16) / 2) as i16
    };
    if prev_avg >= 20 && next_avg + 25 < prev_avg {
        log::warn!(
            "Reject battery jump L/R {}/{} → {}/{} (likely mode ACK noise)",
            prev.left,
            prev.right,
            next.left,
            next.right
        );
        return false;
    }
    // Reject tiny one-sided garbage when other side collapses
    if prev.left >= 20 && next.left > 0 && next.left < 5 {
        return false;
    }
    if prev.right >= 20 && next.right > 0 && next.right < 5 {
        return false;
    }
    true
}

fn should_accept_case(prev: &BatteryState, next: &BatteryState) -> bool {
    if next.case == 0 && !next.case_charging {
        return false;
    }
    if next.case > 100 {
        return false;
    }
    if prev.case >= 20 && next.case > 0 && next.case + 25 < prev.case {
        log::warn!(
            "Reject case battery jump {} → {} (likely noise)",
            prev.case,
            next.case
        );
        return false;
    }
    true
}

async fn apply_event(app: &AppHandle, event: DeviceEvent) {
    match &event {
        DeviceEvent::Battery(partial) => {
            let mut state = BLE.lock().await;
            let prev = state.battery.clone();
            let looks_like_case = partial.left == 0
                && partial.right == 0
                && (partial.case != 0 || partial.case_charging);
            let looks_like_buds = partial.left != 0 || partial.right != 0;

            let mut changed = false;

            if looks_like_case && should_accept_case(&prev, partial) {
                if state.battery.case != partial.case
                    || state.battery.case_charging != partial.case_charging
                {
                    state.battery.case = partial.case;
                    state.battery.case_charging = partial.case_charging;
                    changed = true;
                }
            } else if looks_like_buds && should_accept_buds(&prev, partial) {
                // Merge per-side: keep previous if new side is 0 (partial report)
                let new_l = if partial.left > 0 {
                    partial.left
                } else {
                    state.battery.left
                };
                let new_r = if partial.right > 0 {
                    partial.right
                } else {
                    state.battery.right
                };
                if state.battery.left != new_l
                    || state.battery.right != new_r
                    || state.battery.left_charging != partial.left_charging
                    || state.battery.right_charging != partial.right_charging
                {
                    state.battery.left = new_l;
                    state.battery.right = new_r;
                    if partial.left > 0 {
                        state.battery.left_charging = partial.left_charging;
                    }
                    if partial.right > 0 {
                        state.battery.right_charging = partial.right_charging;
                    }
                    changed = true;
                }
            } else if looks_like_buds || looks_like_case {
                log::debug!(
                    "Battery update ignored (sticky): got L={} R={} C={} prev L={} R={} C={}",
                    partial.left,
                    partial.right,
                    partial.case,
                    prev.left,
                    prev.right,
                    prev.case
                );
            }

            if !changed {
                return;
            }
            let bat = state.battery.clone();
            log::info!(
                "Battery state L={} R={} case={} (chg L/R/C={}/{}/{})",
                bat.left,
                bat.right,
                bat.case,
                bat.left_charging,
                bat.right_charging,
                bat.case_charging
            );
            drop(state);
            let _ = app.emit("device://battery", &bat);
        }
        DeviceEvent::Anc(mode) => {
            let mut state = BLE.lock().await;
            // Only update last_anc + UI when mode actually changes (avoid flicker)
            let prev = state.last_anc;
            state.last_anc = Some(*mode);
            drop(state);
            if prev != Some(*mode) {
                log::info!("ANC mode → {:?}", mode);
            }
            // Always emit stable string for frontend (not opaque enum shape)
            let s = match mode {
                AncMode::Off => "off",
                AncMode::Anc => "anc",
                AncMode::Transparency => "transparency",
            };
            let _ = app.emit("device://anc", s);
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

/// Apply v2 wrap if current connection needs it (Ultra etc.).
async fn maybe_wrap(data: &[u8]) -> Vec<u8> {
    let use_v2 = BLE.lock().await.use_v2_wrap;
    if use_v2 && data.first() == Some(&0xBA) {
        if let Some(w) = protocol::wrap_ba_command(data) {
            log::info!("wrap_v2 bare {:02X?} → {:02X?}", data, w);
            return w;
        }
    }
    data.to_vec()
}

async fn write_bytes(peripheral: &Peripheral, data: &[u8]) -> Result<(), String> {
    let wire = maybe_wrap(data).await;
    write_raw(peripheral, &wire).await
}

async fn write_command(peripheral: &Peripheral, cmd: protocol::Command) -> Result<(), String> {
    let bare = protocol::encode_command(cmd);
    write_bytes(peripheral, &bare).await
}

async fn write_raw(peripheral: &Peripheral, data: &[u8]) -> Result<(), String> {
    if !peripheral
        .is_connected()
        .await
        .map_err(|e| format!("is_connected: {e}"))?
    {
        return Err("Peripheral disconnected".into());
    }

    let write_order = [
        protocol::uuids::write(),
        protocol::uuids::ccsdk_write(),
    ];
    let chars = peripheral.characteristics();

    let ch = write_order
        .iter()
        .find_map(|u| chars.iter().find(|c| c.uuid == *u))
        .or_else(|| {
            chars.iter().find(|c| {
                c.properties.contains(CharPropFlags::WRITE)
                    || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
            })
        })
        .ok_or("No WRITE characteristic")?;

    // Prefer WithResponse when available (official / elaxptr)
    let write_type = if ch.properties.contains(CharPropFlags::WRITE) {
        WriteType::WithResponse
    } else {
        WriteType::WithoutResponse
    };

    log::info!(
        "Write {:02X?} → {} ({})",
        data,
        ch.uuid,
        if matches!(write_type, WriteType::WithResponse) {
            "with-response"
        } else {
            "without-response"
        }
    );

    let result = peripheral.write(ch, data, write_type).await;
    let result = match result {
        Err(e)
            if matches!(write_type, WriteType::WithResponse)
                && ch.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) =>
        {
            log::warn!("Write WithResponse failed ({e}), retry WithoutResponse");
            peripheral
                .write(ch, data, WriteType::WithoutResponse)
                .await
        }
        other => other,
    };

    result.map_err(|e| format!("Write failed: {e}"))?;

    {
        let mut state = BLE.lock().await;
        state.tx_count = state.tx_count.saturating_add(1);
        state.last_tx_ms = Some(now_ms());
        state.last_tx_hex = Some(hex_encode(data));
        state.write_char = Some(ch.uuid.to_string());
        state.has_write_uuid = true;
    }

    if let Some(app) = app_handle() {
        let _ = app.emit("ble://link", &get_connection_state().await.link);
    }
    Ok(())
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

pub async fn send_anc(mode: AncMode, strength: u8) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_anc(mode, strength);
    log::info!("TX ANC {:?} → {:02X?}", mode, data);
    {
        let mut state = BLE.lock().await;
        // Remember user intent so AA34 "ok" acks map back correctly
        state.last_anc = Some(mode);
        if state.mock {
            drop(state);
            if let Some(app) = app_handle() {
                let s = match mode {
                    AncMode::Off => "off",
                    AncMode::Anc => "anc",
                    AncMode::Transparency => "transparency",
                };
                let _ = app.emit("device://anc", s);
            }
            return Ok(());
        }
    }
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_bytes(&p, &d).await })
    })
    .await
}

pub async fn send_eq(preset: EqPreset) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_eq(preset);
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        if let Some(app) = app_handle() {
            let _ = app.emit("device://eq", &preset);
        }
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_bytes(&p, &d).await })
    })
    .await
}

pub async fn send_game_mode(on: bool) -> Result<(), String> {
    let data = Bp1ProAnc::cmd_set_game_mode(on);
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        if let Some(app) = app_handle() {
            let _ = app.emit("device://game", &on);
        }
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_bytes(&p, &d).await })
    })
    .await
}

pub async fn send_find_buds() -> Result<(), String> {
    let data = Bp1ProAnc::cmd_find_buds();
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return Ok(());
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_bytes(&p, &d).await })
    })
    .await
}

pub async fn send_spatial(mode: protocol::SpatialMode) -> Result<(), String> {
    with_connected_peripheral(|p| {
        Box::pin(async move { write_command(&p, protocol::Command::SetSpatial(mode)).await })
    })
    .await
}

pub async fn send_bass_boost(level: u8) -> Result<(), String> {
    let level = level.min(3);
    with_connected_peripheral(|p| {
        Box::pin(async move { write_command(&p, protocol::Command::SetBassBoost(level)).await })
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
    let state = BLE.lock().await;
    let id = match &state.connected_id {
        Some(id) => id.clone(),
        None => return Ok(()),
    };
    let p = state.peripherals.get(&id).cloned();
    drop(state);

    if let Some(p) = p {
        // Best-effort unsubscribe then disconnect (prevents Windows RO_E_CLOSED on next connect)
        for c in p.characteristics().iter().filter(|c| {
            c.uuid == protocol::uuids::notify()
                || c.uuid == protocol::uuids::ccsdk_notify()
                || c.properties.contains(CharPropFlags::NOTIFY)
        }) {
            let _ = p.unsubscribe(c).await;
        }
        let _ = p.disconnect().await;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let mut state = BLE.lock().await;
    state.connected_id = None;
    state.battery = BatteryState::default();
    state.last_anc = None;
    state.reset_link();
    // Drop cached peripheral so next connect re-resolves a fresh WinRT object
    state.peripherals.remove(&id);
    if let Some(d) = state.devices.get_mut(&id) {
        d.connected = false;
    }
    drop(state);
    emit_connection_state(&app).await;
    let _ = app.emit("ble://disconnected", &id);
    Ok(())
}

/// Send BA02 (and BA27) in bare + 789C forms so Pro and Ultra both respond.
async fn send_battery_queries(peripheral: &Peripheral) -> Result<(), String> {
    let bare = vec![0xBA, 0x02];
    // Primary path respects connection wrap flag
    let _ = write_command(peripheral, protocol::Command::QueryBattery).await;
    tokio::time::sleep(Duration::from_millis(40)).await;
    // Force bare BA02 (Pro / some firmware)
    let _ = write_raw(peripheral, &bare).await;
    // Force 789C-wrapped BA02 (Ultra) even if wrap flag wrong
    if let Some(w) = protocol::wrap_ba_command(&bare) {
        tokio::time::sleep(Duration::from_millis(40)).await;
        let _ = write_raw(peripheral, &w).await;
    }
    // Case battery query (some firmwares need BA27; others push AA27 on lid open)
    let case_q = vec![0xBA, 0x27];
    tokio::time::sleep(Duration::from_millis(30)).await;
    let _ = write_raw(peripheral, &case_q).await;
    if let Some(w) = protocol::wrap_ba_command(&case_q) {
        tokio::time::sleep(Duration::from_millis(30)).await;
        let _ = write_raw(peripheral, &w).await;
    }
    Ok(())
}

/// Explicit battery refresh. Sends BA02/BA27 and waits for AA02/AA27 notify.
pub async fn query_battery() -> Result<BatteryState, String> {
    // Demo mode: return seeded mock values
    {
        let state = BLE.lock().await;
        if state.mock {
            return Ok(state.battery.clone());
        }
        if state.connected_id.is_none() {
            return Err("Not connected".into());
        }
    }

    let before_notify = BLE.lock().await.notify_count;
    let before = BLE.lock().await.battery.clone();

    with_connected_peripheral(|p| {
        Box::pin(async move { send_battery_queries(&p).await })
    })
    .await?;

    // Poll for notify-driven battery update (AA02 can take a few hundred ms)
    for _ in 0..12 {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let state = BLE.lock().await;
        let changed = state.battery.left != before.left
            || state.battery.right != before.right
            || state.battery.case != before.case
            || state.notify_count > before_notify;
        let has_pct =
            state.battery.left != 0 || state.battery.right != 0 || state.battery.case != 0;
        if changed && has_pct {
            return Ok(state.battery.clone());
        }
        // If we got new notifies but still 0/0/0, keep waiting a bit
        if state.notify_count > before_notify + 1 && has_pct {
            return Ok(state.battery.clone());
        }
    }

    // Second burst if first round silent
    let _ = with_connected_peripheral(|p| {
        Box::pin(async move { send_battery_queries(&p).await })
    })
    .await;
    for _ in 0..8 {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let state = BLE.lock().await;
        if state.battery.left != 0 || state.battery.right != 0 || state.battery.case != 0 {
            return Ok(state.battery.clone());
        }
    }

    Ok(BLE.lock().await.battery.clone())
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
            level: String::new(),
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
                hint: None,
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
    drop(state);

    emit_connection_state(&app).await;
    let _ = app.emit("ble://connected", &device);
    let _ = app.emit("device://battery", &bat);

    // Simulate periodic battery notify
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let _ = app2.emit("device://anc", "anc");
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = app2.emit("device://eq", &EqPreset::Balanced);
    });

    Ok(device)
}
