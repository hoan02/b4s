use super::error::{ApiError, ApiErrorCode};
use crate::ble;

#[tauri::command]
pub(crate) async fn ble_check_adapter() -> Result<bool, String> {
    Ok(ble::is_adapter_available().await)
}

#[tauri::command]
pub(crate) async fn ble_start_scan(
    app: tauri::AppHandle,
    mock: Option<bool>,
) -> Result<(), ApiError> {
    let result = if mock.unwrap_or(false) {
        ble::start_mock_scan(app).await
    } else {
        ble::scanning::start_scan(app).await
    };
    result.map_err(ApiError::from)
}

#[tauri::command]
pub(crate) async fn ble_stop_scan(app: tauri::AppHandle) -> Result<(), ApiError> {
    ble::scanning::stop_scan(app).await.map_err(ApiError::from)
}

#[tauri::command]
pub(crate) async fn ble_connect(
    app: tauri::AppHandle,
    device_id: String,
    mock: Option<bool>,
) -> Result<ble::BleDevice, ApiError> {
    if mock.unwrap_or(false) || device_id.starts_with("mock-") {
        ble::mock_connect(app, device_id)
            .await
            .map_err(|error| ApiError::new(ApiErrorCode::ConnectionFailed, error, true))
    } else {
        ble::connection::connect(app, device_id)
            .await
            .map_err(ApiError::from)
    }
}

#[tauri::command]
pub(crate) async fn ble_disconnect(app: tauri::AppHandle) -> Result<(), ApiError> {
    ble::connection::disconnect(app)
        .await
        .map_err(|error| ApiError::new(ApiErrorCode::DisconnectFailed, error, true))
}

#[tauri::command]
pub(crate) async fn ble_get_scan_status() -> Result<ble::ScanStatus, String> {
    Ok(ble::get_scan_status().await)
}

#[tauri::command]
pub(crate) async fn ble_get_connection() -> Result<ble::ConnectionState, String> {
    Ok(ble::get_connection_state().await)
}

#[tauri::command]
pub(crate) async fn ble_get_link_health() -> Result<ble::LinkHealth, String> {
    Ok(ble::get_link_health().await)
}
