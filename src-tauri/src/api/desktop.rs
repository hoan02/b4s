use tauri::AppHandle;

#[tauri::command]
pub(crate) fn set_experimental_mode(enabled: bool) {
    crate::device::capability::set_experimental_mode(enabled);
}

#[tauri::command]
pub(crate) fn get_start_at_login(app: AppHandle) -> Result<bool, String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        return app
            .autolaunch()
            .is_enabled()
            .map_err(|error| error.to_string());
    }
    #[cfg(not(desktop))]
    {
        let _ = app;
        Ok(false)
    }
}

#[tauri::command]
pub(crate) fn set_start_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        return if enabled {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|error| error.to_string());
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, enabled);
        Err("Start at login is only available on desktop".into())
    }
}
