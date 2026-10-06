use crate::ble::{self, ConnectionState};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};

struct TrayStatus(MenuItem<Wry>);

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
    let quit = MenuItem::with_id(app, "quit", "Quit B4S", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &status, &separator, &quit])?;

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

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
