use crate::{ble, catalog, device, protocol};
use protocol::{BatteryState, EqBand, ListeningCommand, SpatialMode};

#[tauri::command]
pub(crate) fn list_models() -> Vec<protocol::ModelInfo> {
    protocol::catalog_json()
}

#[tauri::command]
pub(crate) fn list_model_profiles() -> Vec<catalog::ModelProfile> {
    catalog::all_profiles()
}

#[tauri::command]
pub(crate) fn get_model_profile(model_id: String) -> Result<catalog::ModelProfile, String> {
    catalog::profile_for(&model_id).ok_or_else(|| format!("No profile for model: {model_id}"))
}

#[tauri::command]
pub(crate) async fn get_device_snapshot() -> device::snapshot::DeviceSnapshot {
    ble::commands::get_device_snapshot().await
}

#[tauri::command]
pub(crate) async fn query_battery() -> Result<BatteryState, String> {
    ble::connection::query_battery().await
}

#[tauri::command]
pub(crate) async fn set_listening_state(
    mode: String,
    transparency_mode: String,
    adaptive: bool,
    environment: u16,
    level: u8,
) -> Result<(), String> {
    let command = match mode.to_lowercase().as_str() {
        "off" | "normal" => ListeningCommand::Normal,
        "transparency" | "ambient" => match transparency_mode.as_str() {
            "full" => ListeningCommand::TransparencyFull,
            "voice" => ListeningCommand::TransparencyVoice,
            _ => return Err(format!("Unknown transparency mode: {transparency_mode}")),
        },
        "anc" | "noiseReduction" | "noisereduction" => {
            if adaptive {
                ListeningCommand::AdaptiveEnvironment(environment)
            } else {
                ListeningCommand::CustomLevel(level)
            }
        }
        _ => return Err(format!("Unknown listening mode: {mode}")),
    };
    ble::commands::send_listening(command).await
}

#[tauri::command]
pub(crate) async fn set_eq_preset(preset: String) -> Result<(), String> {
    ble::commands::send_eq_id(&preset).await
}

#[tauri::command]
pub(crate) async fn set_eq_index(index: u8) -> Result<(), String> {
    ble::commands::send_eq_index(index).await
}

#[tauri::command]
pub(crate) async fn set_game_mode(enabled: bool) -> Result<(), String> {
    ble::commands::send_game_mode(enabled).await
}

#[tauri::command]
pub(crate) async fn set_spatial_mode(mode: String) -> Result<(), String> {
    let mode = match mode.to_lowercase().as_str() {
        "music" | "01" => SpatialMode::Music,
        "cinema" | "movie" | "02" => SpatialMode::Cinema,
        "game" | "03" => SpatialMode::Game,
        "off" | "00" => SpatialMode::Off,
        _ => return Err(format!("Unknown spatial mode: {mode}")),
    };
    ble::commands::send_spatial(mode).await
}

#[tauri::command]
pub(crate) async fn set_bass_boost(level: u8) -> Result<(), String> {
    ble::commands::send_bass_boost(level).await
}

#[tauri::command]
pub(crate) async fn set_custom_eq(
    bands: Vec<EqBand>,
    dict_sort: u8,
    anc: bool,
) -> Result<(), String> {
    ble::commands::send_custom_eq(bands, dict_sort, anc).await
}

#[tauri::command]
pub(crate) async fn set_ldac(enabled: bool) -> Result<(), String> {
    ble::commands::send_ldac(enabled).await
}

#[tauri::command]
pub(crate) async fn set_hearing_protection(enabled: bool, level: u8) -> Result<(), String> {
    ble::commands::send_hearing_protection(enabled, level).await
}

#[tauri::command]
pub(crate) async fn find_buds(start: bool) -> Result<(), String> {
    ble::commands::send_find_buds(start).await
}
