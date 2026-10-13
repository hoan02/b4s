use super::*;

fn listening_state_parameter(
    family: protocol::ProtocolFamily,
    command: ListeningCommand,
    wire: u8,
) -> u8 {
    if family == protocol::ProtocolFamily::Bp1Ultra {
        match command {
            ListeningCommand::TransparencyFull => return 0xFF,
            ListeningCommand::TransparencyVoice => return 1,
            _ => {}
        }
    }
    wire
}

pub async fn send_listening(command: ListeningCommand) -> Result<(), CommandError> {
    let profile = {
        let state = BLE.lock().await;
        let id = state.connected_id.as_ref().ok_or("Not connected")?;
        state
            .devices
            .get(id)
            .map(|device| device.device_profile.clone())
            .ok_or("Connected device profile is missing")?
    };
    let data = protocol::encode_listening(&profile, command)?;
    let mode = match command {
        ListeningCommand::Normal => AncMode::Off,
        ListeningCommand::TransparencyFull | ListeningCommand::TransparencyVoice => {
            AncMode::Transparency
        }
        ListeningCommand::CustomLevel(_) | ListeningCommand::AdaptiveEnvironment(_) => AncMode::Anc,
    };
    let wire_parameter = data
        .get(3)
        .copied()
        .ok_or("Encoded listening command is incomplete")?;
    let parameter = listening_state_parameter(profile.protocol, command, wire_parameter);
    log::info!("TX ANC {:?} → {:02X?}", mode, data);
    if BLE.lock().await.mock {
        BLE.lock().await.last_anc = Some(mode);
        return observe_mock_state(DeviceEvent::Anc { mode, parameter }, 0x33).await;
    }
    // NoiseReduceManger(2).s: AA34 01 is only success; the app then sends
    // BA33 and reads the state from AA33. A rejection fails fast.
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move {
            let expected = crate::device::confirmation::ExpectedState::Anc { mode, parameter };
            write_and_readback(&p, &d, &[0xBA, 0x33], expected).await
        })
    })
    .await
}

pub async fn send_eq_id(id: &str) -> Result<(), CommandError> {
    let model_id = {
        let state = BLE.lock().await;
        state
            .connected_id
            .as_ref()
            .and_then(|id| state.devices.get(id))
            .and_then(|device| device.model_id.clone())
    }
    .ok_or("Connected model is missing")?;
    let eq = crate::catalog::profile_for(&model_id)
        .and_then(|profile| profile.eq)
        .ok_or("No model EQ schema")?;
    let preset = eq
        .presets
        .iter()
        .find(|preset| preset.id == id)
        .ok_or("Preset ID is absent from the model schema")?;
    send_eq_index(preset.dict_sort).await
}

#[allow(dead_code)]
pub async fn send_eq(preset: EqPreset) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetEq(preset)).await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::Eq(preset), 0x30).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move {
            write_and_readback(
                &p,
                &d,
                &[0xBA, 0x30],
                crate::device::confirmation::ExpectedState::Eq(preset),
            )
            .await
        })
    })
    .await
}

pub async fn send_game_mode(on: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetGameMode(on)).await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::GameMode(on), 0x23).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move {
            write_and_readback(
                &p,
                &d,
                &[0xBA, 0x23],
                crate::device::confirmation::ExpectedState::Game(on),
            )
            .await
        })
    })
    .await
}

pub async fn send_find_buds(side: protocol::FindSide, start: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::FindBuds { side, start }).await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return Ok(());
    }
    drop(state);
    let result = with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move { write_bytes(&p, &d).await })
    })
    .await;
    if result.is_ok() {
        BLE.lock().await.find_requested = start.then_some(side);
    }
    result
}

pub async fn shutdown(app: AppHandle) {
    let _ = tokio::time::timeout(Duration::from_secs(1), scanning::stop_scan(app.clone())).await;
    let started_side = BLE.lock().await.find_requested;
    if let Some(side) = started_side {
        let _ = tokio::time::timeout(Duration::from_secs(1), send_find_buds(side, false)).await;
    }
    let _ = tokio::time::timeout(Duration::from_secs(2), connection::disconnect(app)).await;
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        super::discovery::stop_central_listener(),
    )
    .await;
}

pub async fn send_spatial(mode: protocol::SpatialMode) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetSpatial(mode)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::SpatialMode(mode), 0x42).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x42],
                crate::device::confirmation::ExpectedState::SpatialMode(mode),
            )
            .await
        })
    })
    .await
}

pub async fn send_eq_index(index: u8) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetEqIndex(index)).await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::EqIndex(index), 0x30).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x30],
                crate::device::confirmation::ExpectedState::EqIndex(index),
            )
            .await
        })
    })
    .await
}

pub async fn send_custom_eq(
    bands: Vec<protocol::EqBand>,
    dict_sort: u8,
    anc: bool,
) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetCustomEq {
        dict_sort,
        anc,
        bands,
    })
    .await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::EqIndex(dict_sort), 0x30).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x30],
                crate::device::confirmation::ExpectedState::EqIndex(dict_sort),
            )
            .await
        })
    })
    .await
}

pub async fn send_bass_boost(level: u8) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetBassBoost(level)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::BassBoost(level), 0x53).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x53],
                crate::device::confirmation::ExpectedState::Bass(level),
            )
            .await
        })
    })
    .await
}

pub async fn send_ldac(enabled: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetLdac(enabled)).await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::Ldac(enabled), 0x74).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x74],
                crate::device::confirmation::ExpectedState::Ldac(enabled),
            )
            .await
        })
    })
    .await
}

pub async fn send_hearing_protection(enabled: bool, level: u8) -> Result<(), CommandError> {
    let data =
        encode_connected_feature(protocol::FeatureCommand::SetHearingProtection { enabled, level })
            .await?;
    let state = BLE.lock().await;
    if state.mock {
        drop(state);
        return observe_mock_state(DeviceEvent::HearingProtection { enabled, level }, 0x93).await;
    }
    drop(state);
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x93],
                crate::device::confirmation::ExpectedState::Hearing { enabled, level },
            )
            .await
        })
    })
    .await
}

pub async fn send_in_ear(enabled: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetInEar(enabled)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::InEar(enabled), 0x25).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x25],
                crate::device::confirmation::ExpectedState::InEar(enabled),
            )
            .await
        })
    })
    .await
}

pub async fn send_gesture(
    layout: u8,
    left: Option<u8>,
    right: Option<u8>,
) -> Result<(), CommandError> {
    if BLE.lock().await.gesture_v2 == Some(true) {
        return Err(CommandError::UnsupportedFeature(
            "Firmware uses the V2 gesture contract, which B4S does not support yet".into(),
        ));
    }
    let data = encode_connected_feature(protocol::FeatureCommand::SetGesture {
        layout,
        left,
        right,
    })
    .await?;
    if BLE.lock().await.mock {
        return observe_mock_state(
            DeviceEvent::GestureConfig {
                layout,
                left: left.unwrap_or(0xFF),
                right: right.unwrap_or(0xFF),
            },
            0x21,
        )
        .await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x21, layout],
                crate::device::confirmation::ExpectedState::Gesture {
                    layout,
                    left,
                    right,
                },
            )
            .await
        })
    })
    .await
}

pub async fn send_multipoint(enabled: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetMultipoint(enabled)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::Multipoint(enabled), 0x57).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x57],
                crate::device::confirmation::ExpectedState::Multipoint(enabled),
            )
            .await
        })
    })
    .await
}

pub async fn send_restore_defaults() -> Result<(), CommandError> {
    use crate::device::confirmation::{ExpectedState, RESULT_SUCCESS};
    let data = encode_connected_feature(protocol::FeatureCommand::RestoreDefaults).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::RestoreResult(RESULT_SUCCESS), 0x37).await;
    }
    // EarPhoneSettingActivity/V2: AA37 01 is success (0C dual connection,
    // 0D in a call); HomeBleDataResolvePresenter then re-reads all state.
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_observe(&p, &data, ExpectedState::RestoreResult(RESULT_SUCCESS)).await
        })
    })
    .await?;
    // The re-read is a separate transaction so its duration cannot turn a
    // confirmed restore into a deadline error.
    let requery = with_connected_peripheral(|p| {
        Box::pin(async move {
            let (device, token) = {
                let state = BLE.lock().await;
                let device = state
                    .connected_id
                    .as_ref()
                    .and_then(|id| state.devices.get(id))
                    .cloned();
                (device, state.session.token())
            };
            match device {
                Some(device) => super::connection::run_startup_queries(&p, &device, token).await,
                None => Ok(()),
            }
        })
    })
    .await;
    if let Err(error) = requery {
        log::warn!("State re-query after restore failed: {error}");
    }
    Ok(())
}

pub async fn send_adaptive_lr(enabled: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetAdaptiveLr(enabled)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::AdaptiveLr(enabled), 0x3F).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x3F],
                crate::device::confirmation::ExpectedState::AdaptiveLr(enabled),
            )
            .await
        })
    })
    .await
}

pub async fn send_wind_noise(enabled: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::SetWindNoise(enabled)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::WindNoise(enabled), 0x51).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x51],
                crate::device::confirmation::ExpectedState::WindNoise(enabled),
            )
            .await
        })
    })
    .await
}

async fn encode_connected_feature(command: protocol::FeatureCommand) -> Result<Vec<u8>, String> {
    let state = BLE.lock().await;
    let id = state.connected_id.as_ref().ok_or("Not connected")?;
    let profile = state
        .devices
        .get(id)
        .map(|device| device.device_profile.clone())
        .ok_or("Connected device profile is missing")?;
    protocol::encode_feature(&profile, command)
}

pub async fn get_device_snapshot() -> crate::device::snapshot::DeviceSnapshot {
    BLE.lock().await.snapshot.clone()
}

async fn observe_mock_state(event: DeviceEvent, opcode: u8) -> Result<(), CommandError> {
    let snapshot = {
        let mut state = BLE.lock().await;
        if !state.mock {
            return Err("Connected session is not a demo session".into());
        }
        state.snapshot.observe(opcode, &event, now_ms());
        state.snapshot.clone()
    };
    let app = app_handle().ok_or("Application event handle is unavailable")?;
    app.emit("device://snapshot", &snapshot)
        .map_err(|error| CommandError::Operation(format!("Publish demo snapshot: {error}")))
}

// ---------------------------------------------------------------------------
// Disconnect
// ---------------------------------------------------------------------------

#[cfg(test)]
mod listening_tests {
    use super::*;

    #[test]
    fn ultra_command_expectations_match_normalized_transparency_readbacks() {
        for (command, selector) in [
            (ListeningCommand::TransparencyFull, 1),
            (ListeningCommand::TransparencyVoice, 2),
        ] {
            let frame = protocol::Frame::decode_notify(&[0xAA, 0x33, 2, 102, selector, 5]).unwrap();
            let event =
                protocol::decode_frame(protocol::ProtocolFamily::Bp1Ultra, &frame, None).unwrap();
            assert_eq!(
                event,
                DeviceEvent::Anc {
                    mode: AncMode::Transparency,
                    parameter: listening_state_parameter(
                        protocol::ProtocolFamily::Bp1Ultra,
                        command,
                        selector
                    ),
                }
            );
        }
        assert_eq!(
            listening_state_parameter(
                protocol::ProtocolFamily::Bp1Pro,
                ListeningCommand::TransparencyFull,
                255
            ),
            255
        );
    }
}
