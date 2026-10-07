use super::*;
use crate::device::{DeviceIdentity, DeviceRegistry};

fn insert_scan_device(devices: &mut HashMap<String, BleDevice>, device: BleDevice) -> bool {
    if devices
        .get(&device.id)
        .is_some_and(|existing| device.rssi < existing.rssi)
    {
        return false;
    }
    devices.insert(device.id.clone(), device);
    true
}

fn scan_generation_is_current(scanning: bool, current: u64, event: u64) -> bool {
    scanning && current == event
}

pub async fn start_scan(app: AppHandle) -> Result<(), ScanError> {
    let _scan_operation = super::adapter::SCAN_OPERATION.lock().await;
    if let Err(error) = init_adapter().await {
        log::warn!("Bluetooth adapter unavailable: {error}");
        return Err(ScanError::AdapterUnavailable);
    }
    let adapter = super::adapter::current()
        .await
        .ok_or(ScanError::AdapterUnavailable)?;
    let adapter_state = adapter
        .adapter_state()
        .await
        .map_err(|e| ScanError::Operation(format!("Bluetooth state: {e}")))?;
    if adapter_state == CentralState::PoweredOff {
        return Err(ScanError::BluetoothDisabled);
    }
    let mut state = BLE.lock().await;
    if state.scanning {
        return Ok(());
    }
    let connected = state.connected_id.clone();
    state.devices.retain(|id, _| Some(id.clone()) == connected);
    state
        .peripherals
        .retain(|id, _| Some(id.clone()) == connected);
    state.scanning = true;
    state.scan_generation = state
        .scan_generation
        .checked_add(1)
        .expect("scan generation exhausted");
    state.scan_revision = state.scan_revision.saturating_add(1);
    let scan_generation = state.scan_generation;
    state.mock = false;
    drop(state);

    let start_result: Result<(), ScanError> = async {
        discovery::ensure_central_listener(app.clone(), adapter.clone())
            .await
            .map_err(ScanError::Operation)?;
        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|error| ScanError::Operation(format!("start_scan: {error}")))
    }
    .await;
    if let Err(error) = start_result {
        let mut state = BLE.lock().await;
        if state.scan_generation == scan_generation {
            state.scanning = false;
            state.scan_revision = state.scan_revision.saturating_add(1);
        }
        drop(state);
        emit_scan_status(&app).await;
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

pub(super) async fn process_peripheral(
    app: &AppHandle,
    peripheral: Peripheral,
    id: &PeripheralId,
    scan_generation: u64,
) {
    let event_is_current = {
        let state = BLE.lock().await;
        scan_generation_is_current(state.scanning, state.scan_generation, scan_generation)
    };
    if !event_is_current {
        return;
    }
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
    let device = device_from_identity(
        id_str.clone(),
        rssi,
        advertised_services,
        DeviceIdentity {
            address: address.clone(),
            name: name.clone(),
            advertised_service: props
                .services
                .first()
                .map(|uuid| uuid.to_string().to_uppercase()),
            manufacturer_data: manufacturer.clone(),
        },
    );

    {
        let mut state = BLE.lock().await;
        if !scan_generation_is_current(state.scanning, state.scan_generation, scan_generation) {
            return;
        }
        if !insert_scan_device(&mut state.devices, device) {
            return;
        }

        state.peripherals.insert(id_str, peripheral);
        state.scan_revision = state.scan_revision.saturating_add(1);
    }
    emit_scan_status(app).await;
}

/// Resolve both advertised and OS-enumerated identities through the same reviewed catalog.
pub(super) fn device_from_identity(
    id: String,
    rssi: i16,
    advertised_services: Vec<String>,
    identity: DeviceIdentity,
) -> BleDevice {
    let address = identity.address.clone();
    let name = identity.name.clone();
    let serial = protocol::advertisement::canonical_serial(&identity.manufacturer_data, false);
    let resolved = DeviceRegistry::resolve(identity);
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
    let device_profile = resolved.profile;

    BleDevice {
        id,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceIdentity, DeviceRegistry};

    fn device(id: &str, address: &str, rssi: i16) -> BleDevice {
        let profile = DeviceRegistry::resolve(DeviceIdentity {
            address: address.into(),
            name: "Baseus Bass BP1 Pro".into(),
            advertised_service: None,
            manufacturer_data: Vec::new(),
        })
        .profile;
        BleDevice {
            id: id.into(),
            name: "Baseus Bass BP1 Pro".into(),
            address: address.into(),
            rssi,
            is_baseus: true,
            connected: false,
            headphone_candidate: true,
            model_id: Some("bass-bp1-pro".into()),
            model_name: Some("Baseus Bass BP1 Pro".into()),
            device_profile: profile,
            support: Some("experimental".into()),
            hint: None,
            image_url: None,
            image_provenance: "fallback".into(),
            color_variants: Vec::new(),
            serial: None,
            advertised_services: Vec::new(),
        }
    }

    #[test]
    fn same_address_os_entries_remain_selectable_and_rssi_is_per_entry() {
        let mut devices = HashMap::new();
        assert!(insert_scan_device(
            &mut devices,
            device("audio-entry", "AA:BB:CC:DD:EE:FF", -45)
        ));
        assert!(insert_scan_device(
            &mut devices,
            device("control-entry", "AA:BB:CC:DD:EE:FF", -72)
        ));
        assert_eq!(devices.len(), 2);
        assert_eq!(devices["audio-entry"].rssi, -45);
        assert_eq!(devices["control-entry"].rssi, -72);
        assert!(!insert_scan_device(
            &mut devices,
            device("control-entry", "AA:BB:CC:DD:EE:FF", -80)
        ));
        assert!(insert_scan_device(
            &mut devices,
            device("control-entry", "AA:BB:CC:DD:EE:FF", -60)
        ));
        assert_eq!(devices.len(), 2);
        assert_eq!(devices["control-entry"].rssi, -60);
    }

    #[test]
    fn central_events_cannot_cross_scan_stop_or_restart_boundaries() {
        assert!(scan_generation_is_current(true, 4, 4));
        assert!(!scan_generation_is_current(false, 4, 4));
        assert!(!scan_generation_is_current(true, 5, 4));
    }
}

pub async fn stop_scan(app: AppHandle) -> Result<(), ScanError> {
    stop_scan_session(app, None).await
}

// A previous scan's deadline must not stop a later manual or automatic scan.
async fn stop_scan_session(app: AppHandle, generation: Option<u64>) -> Result<(), ScanError> {
    let _scan_operation = super::adapter::SCAN_OPERATION.lock().await;
    let scan_generation = {
        let state = BLE.lock().await;
        if !state.scanning || generation.is_some_and(|value| value != state.scan_generation) {
            return Ok(());
        }
        state.scan_generation
    };
    let adapter = super::adapter::current().await;

    match adapter {
        Some(adapter) => adapter
            .stop_scan()
            .await
            .map_err(|error| ScanError::Operation(format!("stop_scan: {error}")))?,
        None => {}
    }
    {
        let mut state = BLE.lock().await;
        if !scan_generation_is_current(state.scanning, state.scan_generation, scan_generation) {
            return Ok(());
        }
        state.scanning = false;
        state.scan_revision = state.scan_revision.saturating_add(1);
    }
    emit_scan_status(&app).await;
    Ok(())
}
