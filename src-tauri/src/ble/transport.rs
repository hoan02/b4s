use super::*;

/// Apply v2 wrap if current connection needs it (Ultra etc.).
async fn maybe_wrap(peripheral: &Peripheral, data: &[u8]) -> Result<Vec<u8>, String> {
    let id = id_to_string(&peripheral.id());
    let framing = BLE
        .lock()
        .await
        .devices
        .get(&id)
        .and_then(|device| {
            device
                .device_profile
                .connection
                .as_ref()
                .map(|connection| connection.framing)
        })
        .ok_or("Missing reviewed wire framing")?;
    match framing {
        crate::catalog::WireFraming::BareAaBa => Ok(data.to_vec()),
        crate::catalog::WireFraming::Headphone789c if data.first() == Some(&0xBA) => {
            protocol::wrap_ba_command(data).ok_or("Cannot frame command".into())
        }
        _ => Err("Command framing is unresolved".into()),
    }
}

pub(super) async fn write_bytes(peripheral: &Peripheral, data: &[u8]) -> Result<(), String> {
    let wire = maybe_wrap(peripheral, data).await?;
    write_raw(peripheral, &wire).await
}

pub(super) async fn write_command(
    peripheral: &Peripheral,
    cmd: protocol::Command,
) -> Result<(), String> {
    let bare = protocol::encode_command(cmd);
    write_bytes(peripheral, &bare).await
}

pub(super) async fn write_raw(peripheral: &Peripheral, data: &[u8]) -> Result<(), String> {
    let token = BLE.lock().await.session.token();
    if !peripheral
        .is_connected()
        .await
        .map_err(|e| format!("is_connected: {e}"))?
    {
        return Err("Peripheral disconnected".into());
    }

    let id = id_to_string(&peripheral.id());
    let write_uuid = BLE
        .lock()
        .await
        .devices
        .get(&id)
        .and_then(|device| {
            device
                .device_profile
                .connection
                .as_ref()
                .and_then(|connection| connection.write_uuid.clone())
        })
        .ok_or("Missing reviewed write UUID")?;
    let write_uuid = uuid::Uuid::parse_str(&write_uuid).map_err(|error| error.to_string())?;
    let chars = peripheral.characteristics();
    let ch = chars
        .iter()
        .find(|characteristic| characteristic.uuid == write_uuid)
        .ok_or("Reviewed WRITE characteristic is missing")?;

    if !ch
        .properties
        .intersects(CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE)
    {
        return Err("Reviewed characteristic does not support writes".into());
    }
    // Prefer WithResponse when available (official / elaxptr)
    let write_type = if ch.properties.contains(CharPropFlags::WRITE) {
        WriteType::WithResponse
    } else {
        WriteType::WithoutResponse
    };

    log::info!(
        "Write {:02X?} → {} ({})",
        data,
        ch.uuid,
        if matches!(write_type, WriteType::WithResponse) {
            "with-response"
        } else {
            "without-response"
        }
    );

    ensure_session(token).await?;
    let result = peripheral.write(ch, data, write_type).await;
    result.map_err(|e| format!("Write failed: {e}"))?;

    {
        let mut state = BLE.lock().await;
        if !state.session.accepts(token) {
            return Err("Device session was cancelled during write".into());
        }
        state.diagnostics.record_tx(data, now_ms());
        state.write_char = Some(ch.uuid.to_string());
        state.has_write_uuid = true;
        state.touch_link();
    }

    if let Some(app) = app_handle() {
        let _ = app.emit("ble://link", &get_connection_state().await.link);
    }
    Ok(())
}

pub(super) async fn with_connected_peripheral<F, T>(f: F) -> Result<T, CommandError>
where
    F: FnOnce(
        Peripheral,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
{
    let state = BLE.lock().await;
    if state.mock {
        return Err(CommandError::Operation("MOCK".into()));
    }
    let id = state
        .connected_id
        .as_ref()
        .ok_or(CommandError::NotConnected)?
        .clone();
    let p = state
        .session
        .peripheral()
        .ok_or(CommandError::NotConnected)?;
    let profile = state
        .devices
        .get(&id)
        .map(|device| device.device_profile.clone())
        .ok_or(CommandError::NotConnected)?;
    let token = state.session.token();
    let mut lease = state.session.lease(token);
    let executor = state.session.command_executor();
    drop(state);
    let result = tokio::select! {
        biased;
        _ = lease.cancelled() => Err(CommandError::SessionCancelled),
        result = executor.run(async move {
            crate::device::capability::authorize_control(&profile)
                .map_err(CommandError::UnsupportedFeature)?;
            f(p).await.map_err(CommandError::Operation)
        }) => result.map_err(|error| match error {
            crate::device::executor::ExecutionError::QueueFull => CommandError::QueueFull,
            crate::device::executor::ExecutionError::Deadline => CommandError::Deadline,
            crate::device::executor::ExecutionError::Operation(error) => error,
        }),
    };
    if !BLE.lock().await.session.accepts(token) {
        return Err(CommandError::SessionCancelled);
    }
    result
}

pub(super) async fn write_and_readback(
    peripheral: &Peripheral,
    data: &[u8],
    query: &[u8],
    expected: crate::device::confirmation::ExpectedState,
) -> Result<(), String> {
    let token = BLE.lock().await.session.token();
    let transport = GattConfirmedTransport { peripheral };
    crate::device::confirmation::write_and_confirm(&transport, token, data, query, expected)
        .await?;
    Ok(())
}

pub(super) async fn write_and_observe(
    peripheral: &Peripheral,
    data: &[u8],
    expected: crate::device::confirmation::ExpectedState,
) -> Result<(), String> {
    let token = BLE.lock().await.session.token();
    let transport = GattConfirmedTransport { peripheral };
    crate::device::confirmation::write_and_observe(&transport, token, data, expected).await?;
    Ok(())
}

struct GattConfirmedTransport<'a> {
    peripheral: &'a Peripheral,
}

impl crate::device::confirmation::ConfirmedTransport for GattConfirmedTransport<'_> {
    fn subscribe(
        &self,
    ) -> tokio::sync::broadcast::Receiver<crate::device::confirmation::StateObservation> {
        OBSERVATIONS.subscribe()
    }

    fn write<'a>(
        &'a self,
        payload: &'a [u8],
    ) -> futures::future::BoxFuture<'a, Result<(), String>> {
        Box::pin(write_bytes(self.peripheral, payload))
    }
}
