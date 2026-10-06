// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod ble;
mod catalog;
#[cfg(desktop)]
mod desktop;
mod device;
mod protocol;

use api::{ble::*, desktop::*, device::*, updates::*};
#[cfg(desktop)]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::Manager;

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None,
    ));

    builder
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
            list_model_profiles,
            get_model_profile,
            get_device_snapshot,
            query_battery,
            apply_device_command,
            get_start_at_login,
            set_start_at_login,
            set_experimental_mode,
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

            #[cfg(desktop)]
            {
                let tray_available = match desktop::install_tray(app) {
                    Ok(()) => true,
                    Err(error) => {
                        log::warn!(
                            "System tray unavailable; window close will quit cleanly: {error}"
                        );
                        false
                    }
                };

                if let Some(window) = app.get_webview_window("main") {
                    let close_window = window.clone();
                    let app_handle = app.handle().clone();
                    let close_started = Arc::new(AtomicBool::new(false));
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            if tray_available {
                                api.prevent_close();
                                let _ = close_window.hide();
                            } else {
                                api.prevent_close();
                                if !close_started.swap(true, Ordering::AcqRel) {
                                    desktop::quit(&app_handle);
                                }
                            }
                        }
                    });
                }
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match ble::init_adapter().await {
                    Ok(()) => log::info!("BLE adapter OK"),
                    Err(error) => log::warn!("BLE adapter: {error}"),
                }
                let _ = handle;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
