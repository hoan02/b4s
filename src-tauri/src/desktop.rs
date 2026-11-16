use crate::ble::{self, ConnectionState};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};

struct TrayStatus(MenuItem<Wry>);

/// Argument registered with the OS login entry so a login launch can start hidden.
pub const AUTOSTART_ARG: &str = "--autostart";

pub fn launched_by_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

/// Entries written by older builds lack the argument; rewrite an enabled entry once per launch.
pub fn refresh_autostart_entry(app: &AppHandle) {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(true) => {
            if let Err(error) = manager.enable() {
                log::warn!("Could not refresh the start-at-login entry: {error}");
            }
        }
        Ok(false) => {}
        Err(error) => log::warn!("Could not read the start-at-login entry: {error}"),
    }
}

pub fn install_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open B4S", true, None::<&str>)?;
    let status = MenuItem::with_id(
        app,
        "device-status",
        "Device: disconnected",
        false,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit B4S", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &status, &separator, &quit_item])?;

    app.manage(TrayStatus(status));
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "quit" => quit(app),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

pub fn quit(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = tokio::time::timeout(Duration::from_secs(4), ble::commands::shutdown(app.clone()))
            .await;
        app.exit(0);
    });
}

pub fn update_tray_status(app: &AppHandle, state: &ConnectionState) {
    let Some(status) = app.try_state::<TrayStatus>() else {
        return;
    };
    let label = match (state.connected, state.device.as_ref()) {
        (true, Some(device)) => format!(
            "Connected: {}",
            device.model_name.as_deref().unwrap_or(&device.name)
        ),
        (true, None) => "Device: connected".into(),
        (false, _) => "Device: disconnected".into(),
    };
    let _ = status.0.set_text(label);
}

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
