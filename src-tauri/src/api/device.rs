use crate::{ble, catalog, device, protocol};
use protocol::{BatteryState, EqBand, ListeningCommand, SpatialMode};
use serde::Deserialize;

const DEVICE_COMMAND_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeviceCommandRequest {
    contract_version: u16,
    command: DeviceCommand,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum DeviceCommand {
    SetListeningState {
        mode: ListeningMode,
        transparency_mode: TransparencyMode,
        adaptive: bool,
        environment: u16,
        level: u8,
    },
    SetEqPreset {
        preset: String,
    },
    SetEqIndex {
        index: u8,
    },
    SetCustomEq {
        bands: Vec<EqBand>,
        dict_sort: u8,
        anc: bool,
    },
    SetGameMode {
        enabled: bool,
    },
    SetSpatialMode {
        mode: SpatialMode,
    },
    SetBassBoost {
        level: u8,
    },
    SetLdac {
        enabled: bool,
    },
    SetHearingProtection {
        enabled: bool,
        level: u8,
    },
    FindBuds {
        start: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ListeningMode {
    Off,
    Anc,
    Transparency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TransparencyMode {
    Full,
    Voice,
}

fn validate_device_command_contract(version: u16) -> Result<(), String> {
    if version == DEVICE_COMMAND_CONTRACT_VERSION {
        Ok(())
    } else {
        Err(format!(
            "Unsupported device command contract version: {version}"
        ))
    }
}

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
pub(crate) async fn apply_device_command(request: DeviceCommandRequest) -> Result<(), String> {
    validate_device_command_contract(request.contract_version)?;
    match request.command {
        DeviceCommand::SetListeningState {
            mode,
            transparency_mode,
            adaptive,
            environment,
            level,
        } => {
            let command = match mode {
                ListeningMode::Off => ListeningCommand::Normal,
                ListeningMode::Transparency => match transparency_mode {
                    TransparencyMode::Full => ListeningCommand::TransparencyFull,
                    TransparencyMode::Voice => ListeningCommand::TransparencyVoice,
                },
                ListeningMode::Anc if adaptive => {
                    ListeningCommand::AdaptiveEnvironment(environment)
                }
                ListeningMode::Anc => ListeningCommand::CustomLevel(level),
            };
            ble::commands::send_listening(command).await
        }
        DeviceCommand::SetEqPreset { preset } => ble::commands::send_eq_id(&preset).await,
        DeviceCommand::SetEqIndex { index } => ble::commands::send_eq_index(index).await,
        DeviceCommand::SetCustomEq {
            bands,
            dict_sort,
            anc,
        } => ble::commands::send_custom_eq(bands, dict_sort, anc).await,
        DeviceCommand::SetGameMode { enabled } => ble::commands::send_game_mode(enabled).await,
        DeviceCommand::SetSpatialMode { mode } => ble::commands::send_spatial(mode).await,
        DeviceCommand::SetBassBoost { level } => ble::commands::send_bass_boost(level).await,
        DeviceCommand::SetLdac { enabled } => ble::commands::send_ldac(enabled).await,
        DeviceCommand::SetHearingProtection { enabled, level } => {
            ble::commands::send_hearing_protection(enabled, level).await
        }
        DeviceCommand::FindBuds { start } => ble::commands::send_find_buds(start).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_command_contract_is_versioned_and_closed() {
        let request: DeviceCommandRequest = serde_json::from_value(serde_json::json!({
            "contractVersion": 1,
            "command": { "kind": "setSpatialMode", "mode": "off" }
        }))
        .unwrap();
        assert!(validate_device_command_contract(request.contract_version).is_ok());
        assert!(validate_device_command_contract(0).is_err());
        assert!(
            serde_json::from_value::<DeviceCommandRequest>(serde_json::json!({
                "contractVersion": 1,
                "command": { "kind": "setSpatialMode", "mode": "unknown" }
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<DeviceCommandRequest>(serde_json::json!({
                "contractVersion": 1,
                "command": { "kind": "rawOpcode", "opcode": 0x34 }
            }))
            .is_err()
        );
    }
}
