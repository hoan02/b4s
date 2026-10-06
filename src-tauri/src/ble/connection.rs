use super::*;

async fn resolve_peripheral(device_id: &str) -> Result<Peripheral, String> {
    init_adapter().await?;
    let adapter = {
        let state = BLE.lock().await;
        if let Some(p) = state.peripherals.get(device_id) {
            return Ok(p.clone());
        }
        state
            .adapter
            .as_ref()
            .ok_or("Adapter not initialized")?
            .clone()
    };

    let peris = adapter
        .peripherals()
        .await
        .map_err(|e| format!("list peripherals: {e}"))?;
    for p in peris {
        let id = id_to_string(&p.id());
        if id == device_id {
            let mut state = BLE.lock().await;
            state.peripherals.insert(id, p.clone());
            return Ok(p);
        }
    }
    Err(format!(
        "The selected Bluetooth entry is no longer available; scan again and select it explicitly ({device_id})"
    ))
}

pub async fn connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _attempt = CONNECT_ATTEMPT
        .try_lock()
        .map_err(|_| "Another connection attempt is active")?;
    // Public product metadata supplies recognition, not a command transport.
    // Reject before disconnecting a working device or probing an unknown GATT.
    {
        let state = BLE.lock().await;
        let device = state
            .devices
            .get(&device_id)
            .ok_or("Device is no longer in the scan list")?;
        if device
            .device_profile
            .connection
            .as_ref()
            .map(|connection| connection.transport)
            != Some(crate::catalog::ControlTransport::BleGatt)
        {
            return Err("This model has no reviewed BLE control transport; capture/transport verification is required".into());
        }
        if device.device_profile.protocol == protocol::ProtocolFamily::Unknown {
            return Err(
                "This model is recognized only; its Bluetooth control protocol is not configured"
                    .into(),
            );
        }
    }
    let _ = scanning::stop_scan(app.clone()).await;

    // If already connected to something, disconnect cleanly first
    {
        let state = BLE.lock().await;
        if state.connected_id.is_some() {
            drop(state);
            let _ = disconnect(app.clone()).await;
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    let (attempt_token, mut attempt_lease, session_tasks) = {
        let mut state = BLE.lock().await;
        state.mock = false;
        let session_tasks = state.reset_link();
        let attempt_token = state.session.token();
        let attempt_lease = state.session.lease(attempt_token);
        (attempt_token, attempt_lease, session_tasks)
    };
    super::runtime::join_session_tasks(session_tasks).await;

    let _ = app.emit(
        "ble://connecting",
        &ConnectingState {
            contract_version: 2,
            device_id: device_id.clone(),
            session_id: attempt_token.id(),
        },
    );
    let result = tokio::select! {
        biased;
        _ = attempt_lease.cancelled() => Err("Connection attempt was cancelled".into()),
        result = connect_one(app.clone(), device_id.clone(), attempt_token) => result,
    };
    match result {
        Ok(device) => Ok(device),
        Err(error) => {
            if let Ok(peripheral) = resolve_peripheral(&device_id).await {
                let _ = peripheral.disconnect().await;
            }
            let session_tasks = {
                let mut state = BLE.lock().await;
                if !state.session.accepts(attempt_token) {
                    return Err("Connection attempt was cancelled".into());
                }
                state.connected_id = None;
                let session_tasks = state.reset_link();
                state.peripherals.remove(&device_id);
                session_tasks
            };
            super::runtime::join_session_tasks(session_tasks).await;
            Err(error)
        }
    }
}

async fn connect_one(
    app: AppHandle,
    device_id: String,
    token: crate::device::session::SessionToken,
) -> Result<BleDevice, String> {
    let peripheral = resolve_peripheral(&device_id).await?;
    let mut first_connect = handshake::Handshake::new(30_000);

    // Ensure not half-open
    if peripheral.is_connected().await.unwrap_or(false) {
        let _ = peripheral.disconnect().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    peripheral
        .connect()
        .await
        .map_err(|e| format!("Connect failed: {e}"))?;
    first_connect.on_connected();

    // Small delay then discover — Windows needs this after reconnect
    tokio::time::sleep(Duration::from_millis(250)).await;

    peripheral
        .discover_services()
        .await
        .map_err(|e| format!("Service discovery: {e}"))?;
    first_connect.on_services_discovered();
    ensure_session(token).await?;

    // Keep fresh handle in map
    {
        let mut state = BLE.lock().await;
        state
            .peripherals
            .insert(device_id.clone(), peripheral.clone());
    }

    let device_preview = BLE.lock().await.devices.get(&device_id).cloned();
    let connection = device_preview
        .as_ref()
        .and_then(|device| device.device_profile.connection.clone())
        .ok_or("No reviewed connection profile")?;
    let service_uuid = uuid::Uuid::parse_str(
        connection
            .service_uuid
            .as_deref()
            .ok_or("Missing reviewed service UUID")?,
    )
    .map_err(|error| error.to_string())?;
    if !peripheral
        .services()
        .iter()
        .any(|service| service.uuid == service_uuid)
    {
        return Err("Reviewed control service is missing on this entry".into());
    }
    let write_uuid = uuid::Uuid::parse_str(
        connection
            .write_uuid
            .as_deref()
            .ok_or("Missing reviewed write UUID")?,
    )
    .map_err(|error| error.to_string())?;
    let notify_uuid = uuid::Uuid::parse_str(
        connection
            .notify_uuid
            .as_deref()
            .ok_or("Missing reviewed notify UUID")?,
    )
    .map_err(|error| error.to_string())?;
    let characteristics = peripheral.characteristics();
    let has_write = characteristics.iter().any(|characteristic| {
        characteristic.uuid == write_uuid
            && characteristic
                .properties
                .intersects(CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE)
    });
    let has_notify = characteristics.iter().any(|characteristic| {
        characteristic.uuid == notify_uuid
            && characteristic.properties.contains(CharPropFlags::NOTIFY)
    });
    if !has_write || !has_notify {
        return Err(
            "Reviewed control characteristics are missing or have incompatible properties".into(),
        );
    }
    {
        let mut state = BLE.lock().await;
        state.has_write_uuid = true;
        state.has_notify_uuid = true;
        state.touch_link();
    }
    ensure_session(token).await?;

    // Subscribe to the profile's notify characteristic.
    subscribe_notifications(app.clone(), peripheral.clone(), device_id.clone(), token).await?;
    first_connect.on_notifications_enabled();
    ensure_session(token).await?;

    if connection.init_state_query {
        write_raw(&peripheral, first_connect.init_payload()).await?;
    }
    ensure_session(token).await?;
    // Select the reviewed framing once; OS write completion never chooses it.
    if !connection.handshake.is_empty() {
        write_bytes(&peripheral, &connection.handshake).await?;
    }
    {
        let mut state = BLE.lock().await;
        if !state.session.accepts(token) {
            return Err("Connection attempt was cancelled".into());
        }
        // Diagnostic means write accepted, not device ready.
        state.handshake_ok = !connection.handshake.is_empty();
        state.touch_link();
    }

    // Post-connect battery queries keep the canonical BA02 frame.
    let startup_plan = device_preview
        .as_ref()
        .map(|device| {
            let model = device
                .model_id
                .as_ref()
                .and_then(|id| protocol::catalog_json().into_iter().find(|m| &m.id == id));
            crate::device::initialization::plan_for(model.as_ref(), &device.device_profile)
        })
        .unwrap_or_default();
    for query in startup_plan {
        ensure_session(token).await?;
        if matches!(query, crate::device::initialization::StartupQuery::Battery) {
            let _ = send_battery_queries(&peripheral).await;
        } else if let Some(command) = crate::device::initialization::command_for(query) {
            let _ = write_command(&peripheral, command).await;
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
    // Publish only if this attempt has not been cancelled or superseded.
    let mut state = BLE.lock().await;
    let device = publish_connected_device(&mut state, &device_id, token)?;
    drop(state);

    emit_connection_state(&app).await;
    // Poll battery + link health while connected
    let app_h = app.clone();
    let poll_id = device_id.clone();
    let mut lease = BLE.lock().await.session.lease(token);
    let poller = tokio::spawn(async move {
        for i in 0..40 {
            tokio::select! {
                biased;
                _ = lease.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_secs(if i < 5 { 2 } else { 10 })) => {},
            }
            let still = {
                let s = BLE.lock().await;
                s.session.accepts(token) && s.connected_id.as_ref() == Some(&poll_id)
            };
            if !still {
                break;
            }
            if let Ok(p) = resolve_peripheral(&poll_id).await {
                tokio::select! {
                    biased;
                    _ = lease.cancelled() => break,
                    _ = send_battery_queries(&p) => {},
                }
            }
            emit_connection_state(&app_h).await;
        }
    });
    let mut state = BLE.lock().await;
    if !state
        .session_tasks
        .register_battery_poller(token.id(), poller)
    {
        return Err("Connection session changed before battery polling started".into());
    }
    Ok(device)
}

fn publish_connected_device(
    state: &mut super::runtime::BleInner,
    device_id: &str,
    token: crate::device::session::SessionToken,
) -> Result<BleDevice, String> {
    if !state.session.accepts(token) {
        return Err("Connection attempt was cancelled".into());
    }
    let device = state
        .devices
        .get_mut(device_id)
        .ok_or_else(|| "Selected Bluetooth entry disappeared during connection".to_string())?;
    device.connected = true;
    let device = device.clone();

    state.snapshot.device_id = Some(device_id.to_owned());
    state.snapshot.model_id = device.model_id.clone();
    state.connected_id = Some(device_id.to_owned());
    state.touch_link();
    Ok(device)
}

async fn subscribe_notifications(
    app: AppHandle,
    peripheral: Peripheral,
    device_id: String,
    token: crate::device::session::SessionToken,
) -> Result<(), String> {
    let connection = BLE
        .lock()
        .await
        .devices
        .get(&device_id)
        .and_then(|device| device.device_profile.connection.clone())
        .ok_or("Missing connection profile")?;
    let framing = connection.framing;
    let init_state_query = connection.init_state_query;
    if framing == crate::catalog::WireFraming::Unresolved {
        return Err("Connection framing is unresolved".into());
    }
    let notify_uuid = uuid::Uuid::parse_str(
        connection
            .notify_uuid
            .as_deref()
            .ok_or("Missing notify UUID")?,
    )
    .map_err(|error| error.to_string())?;
    let write_uuid = uuid::Uuid::parse_str(
        connection
            .write_uuid
            .as_deref()
            .ok_or("Missing write UUID")?,
    )
    .map_err(|error| error.to_string())?;
    let chars = peripheral.characteristics();

    let ch = chars
        .iter()
        .find(|characteristic| {
            characteristic.uuid == notify_uuid
                && characteristic.properties.contains(CharPropFlags::NOTIFY)
        })
        .cloned()
        .ok_or_else(|| {
            "Reviewed NOTIFY characteristic is missing or does not support notifications"
                .to_string()
        })?;

    // Retry subscribe once after re-discover (stale handles after disconnect)
    let subscribe_result = peripheral.subscribe(&ch).await;
    let subscribe_result = match subscribe_result {
        Err(e) => {
            log::warn!("Subscribe first try failed ({e}), rediscover + retry");
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = peripheral.discover_services().await;
            let chars2 = peripheral.characteristics();
            let ch2 = chars2
                .iter()
                .find(|c| c.uuid == notify_uuid && c.properties.contains(CharPropFlags::NOTIFY))
                .cloned()
                .ok_or_else(|| format!("Reviewed NOTIFY characteristic disappeared ({e})"))?;
            peripheral.subscribe(&ch2).await.map(|_| ch2)
        }
        Ok(()) => Ok(ch),
    };
    let ch = subscribe_result.map_err(|e| format!("Subscribe failed: {e}"))?;
    log::info!("Subscribed notify {}", ch.uuid);

    let has_write = chars.iter().any(|characteristic| {
        characteristic.uuid == write_uuid
            && characteristic
                .properties
                .intersects(CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE)
    });
    if !has_write {
        return Err("Reviewed WRITE characteristic is missing or incompatible".into());
    }

    {
        let mut state = BLE.lock().await;
        state.notify_char = Some(ch.uuid.to_string());
        state.has_notify_uuid = true;
        state.has_write_uuid = true;
        state.touch_link();
    }

    let mut stream = peripheral
        .notifications()
        .await
        .map_err(|e| format!("notifications stream: {e}"))?;

    let mut lease = BLE.lock().await.session.lease(token);
    let task = tokio::spawn(async move {
        let mut receivers = HashMap::<uuid::Uuid, protocol::receiver::NotificationReceiver>::new();
        loop {
            let n = tokio::select! {
                biased;
                _ = lease.cancelled() => break,
                item = stream.next() => match item { Some(item) => item, None => break },
            };
            if !BLE.lock().await.session.accepts(token) {
                break;
            }
            log::info!("Notify {} : {:02X?}", n.uuid, n.value);
            // Init-state text is a separate handshake message, not an AA frame.
            let is_init_reply = init_state_query
                && std::str::from_utf8(&n.value)
                    .map(|text| {
                        matches!(
                            text.trim().to_ascii_lowercase().as_str(),
                            "init state" | "already configured"
                        )
                    })
                    .unwrap_or(false);
            if is_init_reply {
                handle_notification(&app, &n.value, &device_id, token).await;
                continue;
            }
            let receiver = receivers
                .entry(n.uuid)
                .or_insert_with(|| protocol::receiver::NotificationReceiver::new(framing));
            for frame in receiver.push(&n.value, std::time::Instant::now()) {
                handle_notification(&app, &frame, &device_id, token).await;
            }
        }
        log::info!("Notification stream ended");
    });

    let mut state = BLE.lock().await;
    if !state.session_tasks.register_notification(token.id(), task) {
        return Err(
            "Device session was cancelled before notification ownership was registered".into(),
        );
    }

    Ok(())
}

async fn handle_notification(
    app: &AppHandle,
    data: &[u8],
    device_id: &str,
    token: crate::device::session::SessionToken,
) {
    // Track RX for link health — strongest proof of a real device link
    {
        let mut state = BLE.lock().await;
        if !state.session.accepts(token) {
            return;
        }
        state.diagnostics.record_rx(data, now_ms());
        state.touch_link();
    }
    let _ = app.emit("ble://link", &get_connection_state().await.link);

    let (last_anc, family) = {
        let state = BLE.lock().await;
        if !state.session.accepts(token) {
            return;
        }
        let family = state
            .devices
            .get(device_id)
            .map(|device| device.device_profile.protocol)
            .unwrap_or(protocol::ProtocolFamily::Unknown);
        (state.last_anc, family)
    };

    // The profile-scoped receiver has already decoded only this framing.
    let frames = [data.to_vec()];
    let mut any_decoded = false;
    for frame in &frames {
        match protocol::Frame::decode_notify(frame) {
            Ok(fr) => match protocol::decode_frame(family, &fr, last_anc) {
                Ok(event) => {
                    log::info!("DeviceEvent: {:?}", event);
                    {
                        let mut state = BLE.lock().await;
                        if !state.session.accepts(token) {
                            return;
                        }
                        state.snapshot.device_id = Some(device_id.to_string());
                        state.snapshot.model_id = state
                            .devices
                            .get(device_id)
                            .and_then(|device| device.model_id.clone());
                        state.snapshot.observe(fr.cmd, &event, now_ms());
                        let _ = app.emit("device://snapshot", &state.snapshot);
                        let _ = OBSERVATIONS.send(crate::device::confirmation::StateObservation {
                            session: token,
                            opcode: fr.cmd,
                            event: event.clone(),
                        });
                    }
                    apply_event(event, token, fr.cmd).await;
                    any_decoded = true;
                }
                Err(e) => log::debug!("Decode skip: {e}  raw={:02X?}", frame),
            },
            Err(e) => log::debug!("Frame skip: {e}  raw={:02X?}", frame),
        }
    }

    if !any_decoded {
        log::debug!(
            "Notification did not produce a recognized device state: {:02X?}",
            data
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_scan_entry_cannot_be_published_as_a_synthetic_device() {
        let mut runtime = super::super::runtime::BleInner::new();
        let token = runtime.session.token();

        let result = publish_connected_device(&mut runtime, "disappeared-entry", token);

        assert!(result.is_err());
        assert!(runtime.connected_id.is_none());
        assert!(runtime.snapshot.device_id.is_none());
    }
}

async fn apply_event(event: DeviceEvent, token: crate::device::session::SessionToken, opcode: u8) {
    let mut state = BLE.lock().await;
    if !state.session.accepts(token) {
        return;
    }
    match &event {
        DeviceEvent::Battery(partial) => {
            // Packet identity distinguishes independent reports; zero is a value.
            match opcode {
                0x02 => {
                    state.battery.left = partial.left;
                    state.battery.right = partial.right;
                    state.battery.left_charging = partial.left_charging;
                    state.battery.right_charging = partial.right_charging;
                }
                0x27 => {
                    state.battery.case = partial.case;
                    state.battery.case_charging = partial.case_charging;
                }
                _ => return,
            }
        }
        DeviceEvent::Anc { mode, .. } => {
            // Only update last_anc + UI when mode actually changes (avoid flicker)
            let prev = state.last_anc;
            state.last_anc = Some(*mode);
            if prev != Some(*mode) {
                log::info!("ANC mode → {:?}", mode);
            }
        }
        DeviceEvent::SpatialEnabled(_) => {}
        DeviceEvent::EqIndex(_)
        | DeviceEvent::Eq(_)
        | DeviceEvent::GameMode(_)
        | DeviceEvent::BassBoost(_)
        | DeviceEvent::Ldac(_)
        | DeviceEvent::HearingProtection { .. }
        | DeviceEvent::Unknown { .. } => {}
    }
}

pub async fn disconnect(app: AppHandle) -> Result<(), String> {
    // Invalidate first, including a connection that has not published its ID.
    // No old notification can mutate state during the asynchronous OS cleanup.
    let (id, peripheral, token, session_tasks) = {
        let mut state = BLE.lock().await;
        let id = state.connected_id.take();
        let peripheral = id.as_ref().and_then(|id| state.peripherals.remove(id));
        if let Some(device) = id.as_ref().and_then(|id| state.devices.get_mut(id)) {
            device.connected = false;
        }
        state.battery = BatteryState::default();
        state.last_anc = None;
        let session_tasks = state.reset_link();
        (id, peripheral, state.session.token(), session_tasks)
    };
    super::runtime::join_session_tasks(session_tasks).await;
    if let Some(p) = peripheral {
        let notify_uuid = if let Some(id) = id.as_ref() {
            BLE.lock()
                .await
                .devices
                .get(id)
                .and_then(|device| device.device_profile.connection.as_ref())
                .and_then(|connection| connection.notify_uuid.as_deref())
                .and_then(|value| uuid::Uuid::parse_str(value).ok())
        } else {
            None
        };
        if let Some(notify_uuid) = notify_uuid {
            if let Some(characteristic) = p.characteristics().iter().find(|c| c.uuid == notify_uuid)
            {
                let _ = p.unsubscribe(characteristic).await;
            }
        }
        let _ = p.disconnect().await;
    }
    // A new connection may have started while the OS was cleaning up.
    if BLE.lock().await.session.accepts(token) {
        emit_connection_state(&app).await;
    }
    Ok(())
}

/// Request earbud battery and, for the BP1 family, the independent case report.
async fn send_battery_queries(peripheral: &Peripheral) -> Result<(), String> {
    write_command(peripheral, protocol::Command::QueryBattery).await?;
    let id = id_to_string(&peripheral.id());
    let query_case = {
        let state = BLE.lock().await;
        state.devices.get(&id).is_some_and(|device| {
            device.device_profile.protocol == protocol::ProtocolFamily::Bp1Pro
        })
    };
    if query_case {
        tokio::time::sleep(Duration::from_millis(80)).await;
        // EarFunctionShowPresenter.d uses BA27. Use this model's normal framing;
        // failure to read the case must not discard a valid earbud report.
        if let Err(error) = write_command(peripheral, protocol::Command::QueryCaseBattery).await {
            log::debug!("Case battery query unavailable: {error}");
        }
    }
    Ok(())
}

/// Explicit refresh requests the earbud/case reports and waits for notifications.
pub async fn query_battery() -> Result<BatteryState, String> {
    {
        let state = BLE.lock().await;
        if state.mock {
            return Ok(state.battery.clone());
        }
    }
    with_connected_peripheral(|peripheral| {
        Box::pin(async move {
            let token = BLE.lock().await.session.token();
            let mut replies = OBSERVATIONS.subscribe();
            send_battery_queries(&peripheral).await?;
            let observation = crate::device::confirmation::await_state(
                &mut replies,
                token,
                crate::device::confirmation::ExpectedState::Battery,
            )
            .await?;
            if let DeviceEvent::Battery(mut battery) = observation.event {
                let state = BLE.lock().await;
                battery.case = state.battery.case;
                battery.case_charging = state.battery.case_charging;
                Ok(battery)
            } else {
                Err("Battery readback did not contain battery state".into())
            }
        })
    })
    .await
}

// ---------------------------------------------------------------------------
// Queries / emit
// ---------------------------------------------------------------------------
