use super::error::{ApiError, ApiErrorCode};
use tauri::{AppHandle, Manager};

// Store's initial load ignores parse errors; validate first to preserve corrupt files.
fn validate_settings_file(path: &std::path::Path) -> Result<(), String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&bytes)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_settings_store(app: AppHandle) -> Result<(), ApiError> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| ApiError::new(ApiErrorCode::PreferenceFailed, error.to_string(), false))?
        .join("settings.json");
    validate_settings_file(&path)
        .map_err(|error| ApiError::new(ApiErrorCode::PreferenceFailed, error, false))
}

#[cfg(test)]
mod settings_tests {
    use super::validate_settings_file;

    #[test]
    fn invalid_settings_file_is_preserved_and_missing_file_is_allowed() {
        let path = std::env::temp_dir().join(format!("b4s-settings-{}.json", uuid::Uuid::new_v4()));
        assert!(validate_settings_file(&path).is_ok());
        std::fs::write(&path, b"broken json").unwrap();
        assert!(validate_settings_file(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"broken json");
        std::fs::write(&path, b"[]").unwrap();
        assert!(validate_settings_file(&path).is_err());
        std::fs::write(&path, b"{}").unwrap();
        assert!(validate_settings_file(&path).is_ok());
        std::fs::remove_file(path).unwrap();
    }
}

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
