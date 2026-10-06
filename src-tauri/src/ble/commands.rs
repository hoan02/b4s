use super::*;

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
    let readback = profile.protocol == protocol::ProtocolFamily::Bp1Ultra;
    let data = protocol::encode_listening(&profile, command)?;
    let mode = match command {
        ListeningCommand::Normal => AncMode::Off,
        ListeningCommand::TransparencyFull | ListeningCommand::TransparencyVoice => {
            AncMode::Transparency
        }
        ListeningCommand::CustomLevel(_) | ListeningCommand::AdaptiveEnvironment(_) => AncMode::Anc,
    };
    let parameter = data
        .get(3)
        .copied()
        .ok_or("Encoded listening command is incomplete")?;
    log::info!("TX ANC {:?} → {:02X?}", mode, data);
    if BLE.lock().await.mock {
        BLE.lock().await.last_anc = Some(mode);
        return observe_mock_state(DeviceEvent::Anc { mode, parameter }, 0x34).await;
    }
    with_connected_peripheral(|p| {
        let d = data.clone();
        Box::pin(async move {
            let expected = crate::device::confirmation::ExpectedState::Anc { mode, parameter };
            if readback {
                write_and_readback(&p, &d, &[0xBA, 0x33], expected).await
            } else {
                write_and_observe(&p, &d, expected).await
            }
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

pub async fn send_find_buds(start: bool) -> Result<(), CommandError> {
    let data = encode_connected_feature(protocol::FeatureCommand::FindBuds(start)).await?;
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
        BLE.lock().await.find_requested = start;
    }
    result
}

pub async fn shutdown(app: AppHandle) {
    let _ = tokio::time::timeout(Duration::from_secs(1), scanning::stop_scan(app.clone())).await;
    let should_stop_find = BLE.lock().await.find_requested;
    if should_stop_find {
        let _ = tokio::time::timeout(Duration::from_secs(1), send_find_buds(false)).await;
    }
    let _ = tokio::time::timeout(Duration::from_secs(2), connection::disconnect(app)).await;
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        super::discovery::stop_central_listener(),
    )
    .await;
}

pub async fn send_spatial(mode: protocol::SpatialMode) -> Result<(), CommandError> {
    let ultra = BLE.lock().await.devices.values().any(|device| {
        device.connected && device.device_profile.protocol == protocol::ProtocolFamily::Bp1Ultra
    });
    let data = encode_connected_feature(protocol::FeatureCommand::SetSpatial(mode)).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(
            DeviceEvent::SpatialEnabled(mode != protocol::SpatialMode::Off),
            0x42,
        )
        .await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_readback(
                &p,
                &data,
                &[0xBA, 0x42],
                if ultra {
                    crate::device::confirmation::ExpectedState::SpatialMode(mode)
                } else {
                    crate::device::confirmation::ExpectedState::SpatialEnabled(
                        mode != protocol::SpatialMode::Off,
                    )
                },
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
    let data = encode_connected_feature(protocol::FeatureCommand::RestoreDefaults).await?;
    if BLE.lock().await.mock {
        return observe_mock_state(DeviceEvent::RestoreResult(0), 0x37).await;
    }
    with_connected_peripheral(|p| {
        Box::pin(async move {
            write_and_observe(
                &p,
                &data,
                crate::device::confirmation::ExpectedState::RestoreResult(0),
            )
            .await
        })
    })
    .await
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
