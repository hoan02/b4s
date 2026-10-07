use super::error::{ApiError, ApiErrorCode};
use crate::{ble, catalog, device, protocol};
use protocol::{BatteryState, EqBand, ListeningCommand, SpatialMode};
use serde::{Deserialize, Serialize};

const DEVICE_COMMAND_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeviceCommandRequest {
    contract_version: u16,
    command: DeviceCommand,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
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
    SetInEar {
        enabled: bool,
    },
    SetGesture {
        layout: u8,
        left: Option<u8>,
        right: Option<u8>,
    },
    SetMultipoint {
        enabled: bool,
    },
    RestoreDefaults,
    SetAdaptiveLr {
        enabled: bool,
    },
    SetWindNoise {
        enabled: bool,
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

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum DeviceCommandDisposition {
    DeviceStateObserved,
    TransportAccepted,
    Simulated,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeviceCommandResponse {
    contract_version: u16,
    session_id: u64,
    snapshot_revision: u64,
    disposition: DeviceCommandDisposition,
}

fn validate_device_command_contract(version: u16) -> Result<(), ApiError> {
    if version == DEVICE_COMMAND_CONTRACT_VERSION {
        Ok(())
    } else {
        Err(ApiError::new(
            ApiErrorCode::InvalidRequest,
            format!("Unsupported device command contract version: {version}"),
            false,
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
pub(crate) fn get_model_profile(model_id: String) -> Result<catalog::ModelProfile, ApiError> {
    catalog::profile_for(&model_id).ok_or_else(|| {
        ApiError::new(
            ApiErrorCode::DeviceUnavailable,
            format!("No profile for model: {model_id}"),
            false,
        )
    })
}

#[tauri::command]
pub(crate) async fn get_device_snapshot() -> device::snapshot::DeviceSnapshot {
    ble::commands::get_device_snapshot().await
}

#[tauri::command]
pub(crate) async fn query_battery() -> Result<BatteryState, ApiError> {
    ble::connection::query_battery()
        .await
        .map_err(|error| ApiError::new(ApiErrorCode::BatteryReadFailed, error, true))
}

#[tauri::command]
pub(crate) async fn apply_device_command(
    request: DeviceCommandRequest,
) -> Result<DeviceCommandResponse, ApiError> {
    validate_device_command_contract(request.contract_version)?;
    let disposition = match request.command {
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
            ble::commands::send_listening(command).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetEqPreset { preset } => {
            ble::commands::send_eq_id(&preset).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetEqIndex { index } => {
            ble::commands::send_eq_index(index).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetCustomEq {
            bands,
            dict_sort,
            anc,
        } => {
            ble::commands::send_custom_eq(bands, dict_sort, anc).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetGameMode { enabled } => {
            ble::commands::send_game_mode(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetSpatialMode { mode } => {
            ble::commands::send_spatial(mode).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetBassBoost { level } => {
            ble::commands::send_bass_boost(level).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetLdac { enabled } => {
            ble::commands::send_ldac(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetHearingProtection { enabled, level } => {
            ble::commands::send_hearing_protection(enabled, level).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::FindBuds { start } => {
            ble::commands::send_find_buds(start).await?;
            DeviceCommandDisposition::TransportAccepted
        }
        DeviceCommand::SetInEar { enabled } => {
            ble::commands::send_in_ear(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetGesture {
            layout,
            left,
            right,
        } => {
            ble::commands::send_gesture(layout, left, right).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetMultipoint { enabled } => {
            ble::commands::send_multipoint(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::RestoreDefaults => {
            ble::commands::send_restore_defaults().await?;
            DeviceCommandDisposition::TransportAccepted
        }
        DeviceCommand::SetAdaptiveLr { enabled } => {
            ble::commands::send_adaptive_lr(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
        DeviceCommand::SetWindNoise { enabled } => {
            ble::commands::send_wind_noise(enabled).await?;
            DeviceCommandDisposition::DeviceStateObserved
        }
    };
    let link = ble::get_link_health().await;
    let snapshot = ble::commands::get_device_snapshot().await;
    Ok(DeviceCommandResponse {
        contract_version: DEVICE_COMMAND_CONTRACT_VERSION,
        session_id: snapshot.session_id,
        snapshot_revision: snapshot.revision,
        disposition: if link.mock {
            DeviceCommandDisposition::Simulated
        } else {
            disposition
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_listening_and_custom_eq_use_camel_case_fields() {
        let listening = serde_json::json!({
            "contractVersion": 1,
            "command": {"kind": "setListeningState", "mode": "off", "transparencyMode": "full", "adaptive": true, "environment": 102, "level": 3}
        });
        let request: DeviceCommandRequest = serde_json::from_value(listening.clone()).unwrap();
        assert!(matches!(
            request.command,
            DeviceCommand::SetListeningState {
                transparency_mode: TransparencyMode::Full,
                ..
            }
        ));
        let request: DeviceCommandRequest = serde_json::from_value(serde_json::json!({
            "contractVersion": 1,
            "command": {"kind": "setCustomEq", "dictSort": 101, "anc": false,
                "bands": [{"frequency":100,"qValue":1,"gain":0,"filter":1}]}
        }))
        .unwrap();
        assert!(matches!(
            request.command,
            DeviceCommand::SetCustomEq { dict_sort: 101, .. }
        ));
        let mut legacy = listening;
        legacy["command"]["transparency_mode"] = legacy["command"]["transparencyMode"].take();
        legacy["command"]
            .as_object_mut()
            .unwrap()
            .remove("transparencyMode");
        assert!(serde_json::from_value::<DeviceCommandRequest>(legacy).is_err());
    }

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

    #[test]
    fn command_response_distinguishes_observation_from_transport_acceptance() {
        let accepted = serde_json::to_value(DeviceCommandResponse {
            contract_version: 1,
            session_id: 8,
            snapshot_revision: 21,
            disposition: DeviceCommandDisposition::TransportAccepted,
        })
        .unwrap();
        assert_eq!(accepted["contractVersion"], 1);
        assert_eq!(accepted["sessionId"], 8);
        assert_eq!(accepted["snapshotRevision"], 21);
        assert_eq!(accepted["disposition"], "transportAccepted");

        let observed = serde_json::to_value(DeviceCommandDisposition::DeviceStateObserved).unwrap();
        assert_eq!(observed, "deviceStateObserved");
    }
}
