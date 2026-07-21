// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ble;
mod protocol;

use protocol::{AncMode, BatteryState, EqPreset, SpatialMode};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

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
        ble::start_scan(app).await
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

#[tauri::command]
async fn ble_get_link_health() -> Result<ble::LinkHealth, String> {
    Ok(ble::get_link_health().await)
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
async fn query_battery() -> Result<BatteryState, String> {
    ble::query_battery().await
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
async fn set_spatial_mode(mode: String) -> Result<(), String> {
    let m = match mode.to_lowercase().as_str() {
        "music" | "01" => SpatialMode::Music,
        "cinema" | "movie" | "02" => SpatialMode::Cinema,
        "game" | "03" => SpatialMode::Game,
        _ => SpatialMode::Off,
    };
    ble::send_spatial(m).await
}

#[tauri::command]
async fn set_bass_boost(level: u8) -> Result<(), String> {
    ble::send_bass_boost(level).await
}

#[tauri::command]
async fn find_buds() -> Result<(), String> {
    ble::send_find_buds().await
}

// ---------------------------------------------------------------------------
// App info + updates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: String,
    version: String,
    identifier: String,
    tauri_version: String,
    os: String,
    debug: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckResult {
    available: bool,
    current_version: String,
    version: Option<String>,
    body: Option<String>,
    date: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn get_app_info(app: AppHandle) -> AppInfo {
    let pkg = app.package_info();
    AppInfo {
        name: pkg.name.clone(),
        version: pkg.version.to_string(),
        identifier: app.config().identifier.clone(),
        tauri_version: tauri::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        debug: cfg!(debug_assertions),
    }
}

/// Prefer signed Tauri updater; fall back to GitHub Releases tag compare.
#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<UpdateCheckResult, String> {
    let current = app.package_info().version.to_string();

    // 1) Official Tauri updater (signed latest.json from Releases)
    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                return Ok(UpdateCheckResult {
                    available: true,
                    current_version: current,
                    version: Some(update.version.clone()),
                    body: update.body.clone(),
                    date: update.date.map(|d| d.to_string()),
                    error: None,
                });
            }
            Ok(None) => {
                return Ok(UpdateCheckResult {
                    available: false,
                    current_version: current,
                    version: None,
                    body: None,
                    date: None,
                    error: None,
                });
            }
            Err(e) => {
                log::warn!("Updater check failed, trying GitHub API: {e}");
            }
        },
        Err(e) => {
            log::warn!("Updater unavailable: {e}");
        }
    }

    // 2) Fallback: public GitHub Releases latest tag
    match github_latest_version().await {
        Ok(remote) => {
            let available = is_remote_newer(&remote, &current);
            Ok(UpdateCheckResult {
                available,
                current_version: current,
                version: Some(remote),
                body: if available {
                    Some("Tải bản cài từ GitHub Releases (updater ký chưa sẵn sàng).".into())
                } else {
                    None
                },
                date: None,
                error: None,
            })
        }
        Err(e) => Ok(UpdateCheckResult {
            available: false,
            current_version: current,
            version: None,
            body: None,
            date: None,
            error: Some(e),
        }),
    }
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|e| format!("Updater: {e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("Check update: {e}"))?
        .ok_or_else(|| {
            "Không có bản cập nhật ký số. Mở GitHub Releases để tải thủ công.".to_string()
        })?;

    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|e| format!("Install update: {e}"))?;

    app.restart();
}

async fn github_latest_version() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("B4S-Desktop")
        .build()
        .map_err(|e| e.to_string())?;
    let url = "https://api.github.com/repos/hoan02/b4s/releases/latest";
    let resp = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("GitHub: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub releases HTTP {} (repo public + có release chưa?)",
            resp.status()
        ));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("No tag_name in release")?;
    Ok(tag.trim_start_matches('v').to_string())
}

fn is_remote_newer(remote: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let a = parse(remote);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            ble_check_adapter,
            ble_start_scan,
            ble_stop_scan,
            ble_connect,
            ble_disconnect,
            ble_get_scan_status,
            ble_get_connection,
            ble_get_link_health,
            list_models,
            get_battery,
            query_battery,
            set_anc_mode,
            set_eq_preset,
            set_game_mode,
            set_spatial_mode,
            set_bass_boost,
            find_buds,
            get_app_info,
            check_for_updates,
            install_update,
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
