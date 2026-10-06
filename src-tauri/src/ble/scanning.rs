use super::*;

pub async fn start_scan(app: AppHandle) -> Result<(), String> {
    init_adapter().await?;
    let adapter_state = {
        let state = BLE.lock().await;
        state.adapter.clone()
    }
    .ok_or_else(|| "No Bluetooth adapter found".to_string())?
    .adapter_state()
    .await
    .map_err(|e| format!("Bluetooth state: {e}"))?;
    if adapter_state == CentralState::PoweredOff {
        return Err("Bluetooth đang tắt trên thiết bị này".into());
    }
    let mut state = BLE.lock().await;
    if state.scanning {
        return Ok(());
    }
    let adapter = state
        .adapter
        .as_ref()
        .ok_or("Adapter not initialized")?
        .clone();
    let connected = state.connected_id.clone();
    state.devices.retain(|id, _| Some(id.clone()) == connected);
    state
        .peripherals
        .retain(|id, _| Some(id.clone()) == connected);
    state.scanning = true;
    state.scan_generation = state.scan_generation.wrapping_add(1);
    let scan_generation = state.scan_generation;
    state.mock = false;
    drop(state);

    let start_result = async {
        discovery::ensure_central_listener(app.clone(), adapter.clone()).await?;
        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|error| format!("start_scan: {error}"))
    }
    .await;
    if let Err(error) = start_result {
        let mut state = BLE.lock().await;
        if state.scan_generation == scan_generation {
            state.scanning = false;
        }
        return Err(error);
    }
    emit_scan_status(&app).await;

    let app_t = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let _ = stop_scan_session(app_t, Some(scan_generation)).await;
    });
    Ok(())
}

pub(super) async fn process_peripheral(app: &AppHandle, peripheral: Peripheral, id: &PeripheralId) {
    let scan_generation = {
        let state = BLE.lock().await;
        if !state.scanning {
            return;
        }
        state.scan_generation
    };
    let props = match peripheral.properties().await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let name = props.local_name.unwrap_or_default();
    if name.is_empty() {
        return;
    }
    let id_str = id_to_string(id);
    let address = props.address.to_string();
    let rssi = props.rssi.unwrap_or(-100);
    let advertised_services: Vec<String> = props
        .services
        .iter()
        .map(|uuid| uuid.to_string().to_uppercase())
        .collect();
    let manufacturer: Vec<u8> = props
        .manufacturer_data
        .values()
        .flat_map(|bytes| bytes.iter().copied())
        .collect();
    let resolved = DeviceRegistry::resolve(DeviceIdentity {
        address: address.clone(),
        name: name.clone(),
        advertised_service: advertised_services.first().cloned(),
        manufacturer_data: manufacturer.clone(),
    });
    let is_baseus = resolved.model.is_some();
    let model_id = resolved.model.as_ref().map(|model| model.id.clone());
    let model_name = resolved
        .model
        .as_ref()
        .map(|model| model.display_name.clone());
    let support = resolved.model.as_ref().map(|model| match model.support {
        protocol::SupportLevel::Verified => "verified".into(),
        protocol::SupportLevel::Experimental => "experimental".into(),
        protocol::SupportLevel::ScanOnly => "scanOnly".into(),
    });
    let (image_url, image_provenance, color_variants) = model_presentation(model_id.as_deref());
    let serial = protocol::advertisement::canonical_serial(&manufacturer, false);
    let device_profile = resolved.profile;

    let device = BleDevice {
        id: id_str.clone(),
        name: name.clone(),
        address: address.clone(),
        rssi,
        is_baseus,
        connected: false,
        headphone_candidate: resolved
            .model
            .as_ref()
            .map(|model| crate::catalog::public::is_headphone_candidate(&model.display_name))
            .unwrap_or(true),
        model_id,
        model_name,
        device_profile,
        support,
        hint: None,
        image_url,
        image_provenance,
        color_variants,
        serial,
        advertised_services,
    };

    {
        let mut state = BLE.lock().await;
        if !state.scanning || state.scan_generation != scan_generation {
            return;
        }
        // Prefer stronger RSSI if same Windows id already seen
        if let Some(old) = state.devices.get(&id_str) {
            if rssi < old.rssi {
                // keep stronger
                return;
            }
        }
        // Same MAC already listed under another id → keep stronger, drop weaker
        let addr_key = normalize_addr(&address);
        if !addr_key.is_empty() && addr_key != "00:00:00:00:00:00" {
            let mut drop_ids = Vec::new();
            for (oid, od) in state.devices.iter() {
                if oid != &id_str && normalize_addr(&od.address) == addr_key {
                    if rssi >= od.rssi {
                        drop_ids.push(oid.clone());
                    } else {
                        return; // existing is stronger
                    }
                }
            }
            for oid in drop_ids {
                state.devices.remove(&oid);
                state.peripherals.remove(&oid);
            }
        }

        // Distinct OS entries stay independent so connection always follows the user's selection.
        state.devices.insert(id_str.clone(), device.clone());
        state.peripherals.insert(id_str, peripheral);
    }
    emit_scan_status(app).await;
}

pub async fn stop_scan(app: AppHandle) -> Result<(), String> {
    stop_scan_session(app, None).await
}

// A previous scan's deadline must not stop a later manual or automatic scan.
async fn stop_scan_session(app: AppHandle, generation: Option<u64>) -> Result<(), String> {
    let mut state = BLE.lock().await;
    if !state.scanning || generation.is_some_and(|value| value != state.scan_generation) {
        return Ok(());
    }
    if let Some(a) = &state.adapter {
        let _ = a.stop_scan().await;
    }
    state.scanning = false;
    drop(state);
    emit_scan_status(&app).await;
    Ok(())
}
