use super::error::{ApiError, ApiErrorCode};
use tauri::AppHandle;

#[tauri::command]
pub(crate) fn set_experimental_mode(enabled: bool) {
    crate::device::capability::set_experimental_mode(enabled);
}

#[tauri::command]
pub(crate) fn get_start_at_login(app: AppHandle) -> Result<bool, ApiError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        return app.autolaunch().is_enabled().map_err(|error| {
            ApiError::new(ApiErrorCode::PreferenceFailed, error.to_string(), false)
        });
    }
    #[cfg(not(desktop))]
    {
        let _ = app;
        Ok(false)
    }
}

#[tauri::command]
pub(crate) fn set_start_at_login(app: AppHandle, enabled: bool) -> Result<(), ApiError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        return if enabled {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|error| ApiError::new(ApiErrorCode::PreferenceFailed, error.to_string(), false));
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, enabled);
        Err(ApiError::new(
            ApiErrorCode::PreferenceFailed,
            "Start at login is only available on desktop",
            false,
        ))
    }
}
