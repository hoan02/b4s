//! Capture Studio — in-app BLE protocol reverse-engineering toolkit.
//!
//! Records TX/RX frames, guided stimulus steps, GATT map, and exports
//! a shareable capture bundle for adding new Baseus models.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Tx, // app → device (write)
    Rx, // device → app (notify)
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureEntry {
    pub id: u64,
    pub ts_ms: u64,
    pub direction: Direction,
    pub hex: String,
    pub raw: Vec<u8>,
    /// Optional human label (guided step or manual note)
    pub label: Option<String>,
    /// Decoded hint if protocol recognised the frame
    pub decoded: Option<String>,
    pub char_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub done: bool,
    pub entry_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GattChar {
    pub uuid: String,
    pub properties: Vec<String>,
    pub service_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSession {
    pub active: bool,
    pub started_at_ms: Option<u64>,
    pub device_name: Option<String>,
    pub device_address: Option<String>,
    pub entries: Vec<CaptureEntry>,
    pub steps: Vec<GuidedStep>,
    pub gatt_map: Vec<GattChar>,
    pub current_step_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureBundle {
    pub version: u32,
    pub exported_at_ms: u64,
    pub device_name: Option<String>,
    pub device_address: Option<String>,
    pub gatt_map: Vec<GattChar>,
    pub entries: Vec<CaptureEntry>,
    pub steps: Vec<GuidedStep>,
    pub notes: String,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct CaptureInner {
    active: bool,
    started_at_ms: Option<u64>,
    device_name: Option<String>,
    device_address: Option<String>,
    entries: Vec<CaptureEntry>,
    steps: Vec<GuidedStep>,
    gatt_map: Vec<GattChar>,
    current_step_id: Option<String>,
    max_entries: usize,
}

impl CaptureInner {
    fn new() -> Self {
        Self {
            active: false,
            started_at_ms: None,
            device_name: None,
            device_address: None,
            entries: Vec::new(),
            steps: default_guided_steps(),
            gatt_map: Vec::new(),
            current_step_id: None,
            max_entries: 5000,
        }
    }
}

static CAPTURE: Lazy<Arc<Mutex<CaptureInner>>> =
    Lazy::new(|| Arc::new(Mutex::new(CaptureInner::new())));
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static CAPTURE_ON: AtomicBool = AtomicBool::new(false);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_guided_steps() -> Vec<GuidedStep> {
    vec![
        GuidedStep {
            id: "connect".into(),
            title: "1. Connect",
            instruction: "Pair and connect the earbuds. Watch for identity / handshake frames.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "battery".into(),
            title: "2. Battery",
            instruction: "Open the case lid (or put buds in/out). Capture battery notify frames.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "anc_off".into(),
            title: "3. ANC Off",
            instruction: "Set ANC to Off from the control UI (or write BA 34 00 FF).".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "anc_on".into(),
            title: "4. ANC On",
            instruction: "Enable ANC. Note mode byte + strength level in TX and ack in RX.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "transparency".into(),
            title: "5. Transparency",
            instruction: "Switch to Ambient / Transparency mode.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "eq".into(),
            title: "6. EQ presets",
            instruction: "Cycle EQ: Balanced → Bass → Voice → Clear. Mark each TX/RX pair.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "game".into(),
            title: "7. Game mode",
            instruction: "Toggle Game / low-latency mode ON then OFF.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "find".into(),
            title: "8. Find buds",
            instruction: "Trigger Find My Buds if available.".into(),
            done: false,
            entry_ids: vec![],
        },
        GuidedStep {
            id: "disconnect".into(),
            title: "9. Disconnect",
            instruction: "Close case or disconnect. Capture teardown frames.".into(),
            done: false,
            entry_ids: vec![],
        },
    ]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn is_active() -> bool {
    CAPTURE_ON.load(Ordering::Relaxed)
}

pub async fn start(
    app: AppHandle,
    device_name: Option<String>,
    device_address: Option<String>,
) -> Result<CaptureSession, String> {
    let mut inner = CAPTURE.lock().await;
    inner.active = true;
    inner.started_at_ms = Some(now_ms());
    inner.device_name = device_name;
    inner.device_address = device_address;
    inner.entries.clear();
    inner.steps = default_guided_steps();
    inner.current_step_id = Some("connect".into());
    CAPTURE_ON.store(true, Ordering::Relaxed);

    // Info entry
    let entry = make_entry(
        Direction::Info,
        &[],
        Some("Capture started".into()),
        None,
        None,
    );
    inner.entries.push(entry);

    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(session)
}

pub async fn stop(app: AppHandle) -> Result<CaptureSession, String> {
    let mut inner = CAPTURE.lock().await;
    inner.active = false;
    CAPTURE_ON.store(false, Ordering::Relaxed);
    let entry = make_entry(
        Direction::Info,
        &[],
        Some(format!("Capture stopped — {} frames", inner.entries.len())),
        None,
        None,
    );
    inner.entries.push(entry);
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(session)
}

pub async fn clear(app: AppHandle) -> Result<CaptureSession, String> {
    let mut inner = CAPTURE.lock().await;
    inner.entries.clear();
    inner.steps.iter_mut().for_each(|s| {
        s.done = false;
        s.entry_ids.clear();
    });
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(session)
}

pub async fn get_session() -> CaptureSession {
    let inner = CAPTURE.lock().await;
    snapshot(&inner)
}

pub async fn set_device_info(name: Option<String>, address: Option<String>) {
    let mut inner = CAPTURE.lock().await;
    if name.is_some() {
        inner.device_name = name;
    }
    if address.is_some() {
        inner.device_address = address;
    }
}

pub async fn set_gatt_map(app: AppHandle, map: Vec<GattChar>) {
    let mut inner = CAPTURE.lock().await;
    inner.gatt_map = map;
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
}

/// Record a TX (write) or RX (notify) frame. No-op if capture inactive.
pub async fn record(
    app: &AppHandle,
    direction: Direction,
    data: &[u8],
    char_uuid: Option<String>,
    decoded: Option<String>,
) {
    if !CAPTURE_ON.load(Ordering::Relaxed) {
        return;
    }

    let mut inner = CAPTURE.lock().await;
    if !inner.active {
        return;
    }

    let label = inner
        .current_step_id
        .as_ref()
        .and_then(|id| inner.steps.iter().find(|s| &s.id == id))
        .map(|s| s.title.clone());

    let entry = make_entry(direction, data, label, decoded, char_uuid);
    let id = entry.id;

    // Attach to current guided step
    if let Some(step_id) = &inner.current_step_id {
        if let Some(step) = inner.steps.iter_mut().find(|s| &s.id == step_id) {
            step.entry_ids.push(id);
        }
    }

    inner.entries.push(entry);
    if inner.entries.len() > inner.max_entries {
        let drain = inner.entries.len() - inner.max_entries;
        inner.entries.drain(0..drain);
    }

    let entry_ref = inner.entries.last().cloned();
    let session = snapshot(&inner);
    drop(inner);

    if let Some(e) = entry_ref {
        let _ = app.emit("capture://entry", &e);
    }
    emit_session(app, &session);
}

pub async fn annotate(app: AppHandle, entry_id: u64, label: String) -> Result<(), String> {
    let mut inner = CAPTURE.lock().await;
    let entry = inner
        .entries
        .iter_mut()
        .find(|e| e.id == entry_id)
        .ok_or("Entry not found")?;
    entry.label = Some(label);
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(())
}

pub async fn add_note(app: AppHandle, text: String) -> Result<(), String> {
    let mut inner = CAPTURE.lock().await;
    let entry = make_entry(Direction::Info, &[], Some(text), None, None);
    inner.entries.push(entry.clone());
    let session = snapshot(&inner);
    drop(inner);
    let _ = app.emit("capture://entry", &entry);
    emit_session(&app, &session);
    Ok(())
}

pub async fn set_current_step(app: AppHandle, step_id: String) -> Result<(), String> {
    let mut inner = CAPTURE.lock().await;
    if !inner.steps.iter().any(|s| s.id == step_id) {
        return Err(format!("Unknown step: {step_id}"));
    }
    inner.current_step_id = Some(step_id.clone());
    let entry = make_entry(
        Direction::Info,
        &[],
        Some(format!("Step → {step_id}")),
        None,
        None,
    );
    inner.entries.push(entry);
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(())
}

pub async fn mark_step_done(app: AppHandle, step_id: String, done: bool) -> Result<(), String> {
    let mut inner = CAPTURE.lock().await;
    let step = inner
        .steps
        .iter_mut()
        .find(|s| s.id == step_id)
        .ok_or("Step not found")?;
    step.done = done;
    let session = snapshot(&inner);
    drop(inner);
    emit_session(&app, &session);
    Ok(())
}

pub async fn export_bundle(notes: Option<String>) -> Result<CaptureBundle, String> {
    let inner = CAPTURE.lock().await;
    Ok(CaptureBundle {
        version: 1,
        exported_at_ms: now_ms(),
        device_name: inner.device_name.clone(),
        device_address: inner.device_address.clone(),
        gatt_map: inner.gatt_map.clone(),
        entries: inner.entries.clone(),
        steps: inner.steps.clone(),
        notes: notes.unwrap_or_default(),
    })
}

/// Export as markdown packet-table draft.
pub async fn export_markdown() -> Result<String, String> {
    let inner = CAPTURE.lock().await;
    let mut md = String::new();
    md.push_str("# Capture Bundle\n\n");
    md.push_str(&format!(
        "- **Device:** {}\n",
        inner.device_name.as_deref().unwrap_or("Unknown")
    ));
    md.push_str(&format!(
        "- **Address:** {}\n",
        inner.device_address.as_deref().unwrap_or("—")
    ));
    md.push_str(&format!("- **Frames:** {}\n", inner.entries.len()));
    md.push_str(&format!(
        "- **Exported:** {}\n\n",
        now_ms()
    ));

    if !inner.gatt_map.is_empty() {
        md.push_str("## GATT map\n\n");
        md.push_str("| Service | Characteristic | Properties |\n");
        md.push_str("|---------|----------------|------------|\n");
        for c in &inner.gatt_map {
            md.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                c.service_uuid,
                c.uuid,
                c.properties.join(", ")
            ));
        }
        md.push('\n');
    }

    md.push_str("## Frames\n\n");
    md.push_str("| # | Dir | Hex | Label | Decoded |\n");
    md.push_str("|---|-----|-----|-------|----------|\n");
    for (i, e) in inner.entries.iter().enumerate() {
        if e.direction == Direction::Info && e.hex.is_empty() {
            md.push_str(&format!(
                "| {} | INFO | — | {} | |\n",
                i + 1,
                e.label.as_deref().unwrap_or("")
            ));
            continue;
        }
        let dir = match e.direction {
            Direction::Tx => "TX",
            Direction::Rx => "RX",
            Direction::Info => "INFO",
        };
        md.push_str(&format!(
            "| {} | {} | `{}` | {} | {} |\n",
            i + 1,
            dir,
            e.hex,
            e.label.as_deref().unwrap_or(""),
            e.decoded.as_deref().unwrap_or("")
        ));
    }

    md.push_str("\n## Guided steps\n\n");
    for s in &inner.steps {
        let mark = if s.done { "x" } else { " " };
        md.push_str(&format!(
            "- [{}] **{}** — {} ({} frames)\n",
            mark,
            s.title,
            s.instruction,
            s.entry_ids.len()
        ));
    }

    md.push_str("\n---\n*Generated by Baseus Desktop Capture Studio*\n");
    Ok(md)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_entry(
    direction: Direction,
    data: &[u8],
    label: Option<String>,
    decoded: Option<String>,
    char_uuid: Option<String>,
) -> CaptureEntry {
    CaptureEntry {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        ts_ms: now_ms(),
        direction,
        hex: if data.is_empty() {
            String::new()
        } else {
            hex_encode(data)
        },
        raw: data.to_vec(),
        label,
        decoded,
        char_uuid,
    }
}

fn snapshot(inner: &CaptureInner) -> CaptureSession {
    CaptureSession {
        active: inner.active,
        started_at_ms: inner.started_at_ms,
        device_name: inner.device_name.clone(),
        device_address: inner.device_address.clone(),
        entries: inner.entries.clone(),
        steps: inner.steps.clone(),
        gatt_map: inner.gatt_map.clone(),
        current_step_id: inner.current_step_id.clone(),
    }
}

fn emit_session(app: &AppHandle, session: &CaptureSession) {
    let _ = app.emit("capture://session", session);
}

/// Try to produce a short decoded hint from known BP1 opcodes.
pub fn hint_decode(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }
    let magic = data[0];
    let cmd = data[1];
    let payload = &data[2..];

    let dir = if magic == 0xBA {
        "TX"
    } else if magic == 0xAA {
        "RX"
    } else {
        return Some(format!("raw magic={magic:#04x}"));
    };

    let meaning = match cmd {
        0x02 if payload.len() >= 4 => {
            format!("Battery L={}% R={}%", payload[0], payload[2])
        }
        0x23 => format!(
            "GameMode {}",
            if payload.first().copied().unwrap_or(0) != 0 {
                "ON"
            } else {
                "OFF"
            }
        ),
        0x24 => format!(
            "GameMode cmd {}",
            if payload.first().copied().unwrap_or(0) != 0 {
                "ON"
            } else {
                "OFF"
            }
        ),
        0x27 if !payload.is_empty() => {
            format!(
                "Case {}% chg={}",
                payload[0],
                payload.get(1).copied().unwrap_or(0)
            )
        }
        0x34 if payload.len() >= 2 => {
            let mode = match payload[0] {
                0 => "Off",
                1 => "ANC",
                2 => "Transparency",
                _ => "?",
            };
            format!("ANC {mode} level={:#04x}", payload[1])
        }
        0x34 => "ANC ack".into(),
        0x42 => format!("EQ query resp {}", payload.first().unwrap_or(&0)),
        0x43 => {
            let names = ["Balanced", "Bass", "Voice", "Clear"];
            let i = payload.first().copied().unwrap_or(0) as usize;
            format!("EQ {}", names.get(i).unwrap_or(&"?"))
        }
        0x12 => "Identity".into(),
        0x30 => "Keepalive".into(),
        0x32 => "ANC Transparency notify".into(),
        0x33 => "ANC On notify".into(),
        0x80 => "Case event".into(),
        0x92 => "Find/Gesture?".into(),
        _ => format!("cmd={cmd:#04x}"),
    };

    Some(format!("{dir} {meaning}"))
}
