// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ble;
mod capture;
mod protocol;

use protocol::{AncMode, BatteryState, EqPreset};
use tauri::Manager;

// ---------------------------------------------------------------------------
// BLE commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn ble_check_adapter() -> Result<bool, String> {
    Ok(ble::is_adapter_available().await)
}

#[tauri::command]
async fn ble_start_scan(app: tauri::AppHandle, mock: Option<bool>) -> Result<(), String> {
    if mock.unwrap_or(false) {
        ble::start_mock_scan(app).await
    } else {
        match ble::start_scan(app.clone()).await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::warn!("Real BLE failed ({e}), mock fallback");
                ble::start_mock_scan(app).await
            }
        }
    }
}

#[tauri::command]
async fn ble_stop_scan(app: tauri::AppHandle) -> Result<(), String> {
    ble::stop_scan(app).await
}

#[tauri::command]
async fn ble_connect(
    app: tauri::AppHandle,
    device_id: String,
    mock: Option<bool>,
) -> Result<ble::BleDevice, String> {
    if mock.unwrap_or(false) || device_id.starts_with("mock-") {
        ble::mock_connect(app, device_id).await
    } else {
        ble::connect(app, device_id).await
    }
}

#[tauri::command]
async fn ble_disconnect(app: tauri::AppHandle) -> Result<(), String> {
    ble::disconnect(app).await
}

#[tauri::command]
async fn ble_get_scan_status() -> Result<ble::ScanStatus, String> {
    Ok(ble::get_scan_status().await)
}

#[tauri::command]
async fn ble_get_connection() -> Result<ble::ConnectionState, String> {
    Ok(ble::get_connection_state().await)
}

// ---------------------------------------------------------------------------
// Device control
// ---------------------------------------------------------------------------


#[tauri::command]
fn list_models() -> Vec<protocol::ModelInfo> {
    protocol::catalog_json()
}

#[tauri::command]
async fn get_battery() -> Result<BatteryState, String> {
    Ok(ble::get_battery_state().await)
}

#[tauri::command]
async fn set_anc_mode(mode: String, strength: Option<u8>) -> Result<(), String> {
    let anc = match mode.to_lowercase().as_str() {
        "off" => AncMode::Off,
        "transparency" | "ambient" => AncMode::Transparency,
        _ => AncMode::Anc,
    };
    let strength = strength.unwrap_or(70);
    ble::send_anc(anc, strength).await
}

#[tauri::command]
async fn set_eq_preset(preset: String) -> Result<(), String> {
    let eq = EqPreset::from_ui(&preset);
    ble::send_eq(eq).await
}

#[tauri::command]
async fn set_game_mode(enabled: bool) -> Result<(), String> {
    ble::send_game_mode(enabled).await
}

#[tauri::command]
async fn find_buds() -> Result<(), String> {
    ble::send_find_buds().await
}

// ---------------------------------------------------------------------------
// Capture Studio
// ---------------------------------------------------------------------------

#[tauri::command]
async fn capture_start(
    app: tauri::AppHandle,
    device_name: Option<String>,
    device_address: Option<String>,
) -> Result<capture::CaptureSession, String> {
    capture::start(app, device_name, device_address).await
}

#[tauri::command]
async fn capture_stop(app: tauri::AppHandle) -> Result<capture::CaptureSession, String> {
    capture::stop(app).await
}

#[tauri::command]
async fn capture_clear(app: tauri::AppHandle) -> Result<capture::CaptureSession, String> {
    capture::clear(app).await
}

#[tauri::command]
async fn capture_get() -> Result<capture::CaptureSession, String> {
    Ok(capture::get_session().await)
}

#[tauri::command]
async fn capture_set_step(app: tauri::AppHandle, step_id: String) -> Result<(), String> {
    capture::set_current_step(app, step_id).await
}

#[tauri::command]
async fn capture_mark_step(
    app: tauri::AppHandle,
    step_id: String,
    done: bool,
) -> Result<(), String> {
    capture::mark_step_done(app, step_id, done).await
}

#[tauri::command]
async fn capture_annotate(
    app: tauri::AppHandle,
    entry_id: u64,
    label: String,
) -> Result<(), String> {
    capture::annotate(app, entry_id, label).await
}

#[tauri::command]
async fn capture_add_note(app: tauri::AppHandle, text: String) -> Result<(), String> {
    capture::add_note(app, text).await
}

#[tauri::command]
async fn capture_export_json(notes: Option<String>) -> Result<capture::CaptureBundle, String> {
    capture::export_bundle(notes).await
}

#[tauri::command]
async fn capture_export_markdown() -> Result<String, String> {
    capture::export_markdown().await
}

#[tauri::command]
async fn capture_write_hex(hex: String) -> Result<(), String> {
    ble::write_hex(&hex).await
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // BLE
            ble_check_adapter,
            ble_start_scan,
            ble_stop_scan,
            ble_connect,
            ble_disconnect,
            ble_get_scan_status,
            ble_get_connection,
            // Device
            list_models,
            get_battery,
            set_anc_mode,
            set_eq_preset,
            set_game_mode,
            find_buds,
            // Capture Studio
            capture_start,
            capture_stop,
            capture_clear,
            capture_get,
            capture_set_step,
            capture_mark_step,
            capture_annotate,
            capture_add_note,
            capture_export_json,
            capture_export_markdown,
            capture_write_hex,
        ])
        .setup(|app| {
            ble::set_app_handle(app.handle().clone());

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match ble::init_adapter().await {
                    Ok(()) => log::info!("BLE adapter OK"),
                    Err(e) => log::warn!("BLE adapter: {e}"),
                }
                let _ = handle;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
