//! Explicit demo-mode scanning and connection state.

use super::*;
use crate::device::{DeviceIdentity, DeviceRegistry};

pub async fn start_mock_scan(app: AppHandle) -> Result<(), String> {
    let mut state = BLE.lock().await;
    state.scanning = true;
    state.scan_generation = state
        .scan_generation
        .checked_add(1)
        .expect("scan generation exhausted");
    state.scan_revision = state.scan_revision.saturating_add(1);
    let scan_generation = state.scan_generation;
    state.mock = true;
    state.devices.clear();
    drop(state);
    emit_scan_status(&app).await;

    let mock_names: [(&str, &str, i16); 6] = [
        ("mock-bp1", "Bass BP1 Pro", -42),
        ("mock-ma10", "Baseus Bowie MA10", -51),
        ("mock-ma10s", "Bowie MA10s", -55),
        ("mock-m2s", "Bowie M2s Pro", -60),
        ("mock-e3", "Bowie E3", -63),
        ("mock-inspire", "Inspire XP1", -48),
    ];
    let mocks: Vec<BleDevice> = mock_names
        .iter()
        .map(|(id, name, rssi)| {
            let (is_baseus, model_id, model_name, support) = model_fields(name);
            let (image_url, image_provenance, color_variants) =
                model_presentation(model_id.as_deref());
            let device_profile =
                protocol::profile_for(model_id.as_deref(), model_name.as_deref(), None);
            BleDevice {
                id: (*id).into(),
                name: (*name).into(),
                address: format!("AA:BB:CC:DD:EE:{:02X}", rssi.unsigned_abs() % 200),
                rssi: *rssi,
                is_baseus,
                connected: false,
                headphone_candidate: true,
                model_id,
                model_name,
                device_profile,
                support,
                hint: None,
                image_url,
                image_provenance,
                color_variants,
                serial: None,
                advertised_services: Vec::new(),
            }
        })
        .collect();

    for (i, dev) in mocks.into_iter().enumerate() {
        let app2 = app.clone();
        let d = dev.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(350 * (i as u64 + 1))).await;
            {
                let mut state = BLE.lock().await;
                if !state.scanning || state.scan_generation != scan_generation {
                    return;
                }
                state.devices.insert(d.id.clone(), d);
                state.scan_revision = state.scan_revision.saturating_add(1);
            }
            emit_scan_status(&app2).await;
        });
    }

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        let mut state = BLE.lock().await;
        if state.scan_generation != scan_generation {
            return;
        }
        state.scanning = false;
        state.scan_revision = state.scan_revision.saturating_add(1);
        drop(state);
        emit_scan_status(&app2).await;
    });
    Ok(())
}

pub async fn mock_connect(app: AppHandle, device_id: String) -> Result<BleDevice, String> {
    let _attempt = CONNECT_ATTEMPT
        .try_lock()
        .map_err(|_| "Another connection attempt is active")?;
    let _ = scanning::stop_scan(app.clone()).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let (device, previous_peripheral, session_tasks) = {
        let mut state = BLE.lock().await;
        let mut device = state
            .devices
            .get(&device_id)
            .ok_or("Mock device not found")?
            .clone();
        let previous_peripheral = state.session.take_peripheral();
        let session_tasks = state.reset_link();
        state.mock = true;
        state.has_write_uuid = false;
        state.has_notify_uuid = false;
        state.handshake_ok = false;
        state.diagnostics = Default::default();
        device.connected = true;
        state.connected_id = Some(device_id);
        if let Some(current) = state.devices.get_mut(&device.id) {
            current.connected = true;
        }
        state.battery = BatteryState {
            left: 87,
            right: 92,
            case: 64,
            left_charging: false,
            right_charging: false,
            case_charging: true,
        };
        let battery = state.battery.clone();
        state.snapshot.device_id = Some(device.id.clone());
        state.snapshot.model_id = device.model_id.clone();
        state.snapshot.mock = true;
        state
            .snapshot
            .observe(2, &DeviceEvent::Battery(battery.clone()), now_ms());
        state
            .snapshot
            .observe(0x27, &DeviceEvent::Battery(battery), now_ms());
        state.snapshot.observe(
            0x34,
            &DeviceEvent::Anc {
                mode: AncMode::Anc,
                parameter: 0xFF,
            },
            now_ms(),
        );
        state
            .snapshot
            .observe(0x42, &DeviceEvent::Eq(EqPreset::Balanced), now_ms());
        state
            .snapshot
            .observe(0x23, &DeviceEvent::GameMode(false), now_ms());
        (device, previous_peripheral, session_tasks)
    };
    super::runtime::join_session_tasks(session_tasks).await;
    if let Some(peripheral) = previous_peripheral {
        let _ = peripheral.disconnect().await;
    }

    emit_connection_state(&app).await;
    let snapshot = BLE.lock().await.snapshot.clone();
    let _ = app.emit("device://snapshot", &snapshot);
    Ok(device)
}

fn model_fields(name: &str) -> (bool, Option<String>, Option<String>, Option<String>) {
    let resolved = DeviceRegistry::resolve(DeviceIdentity {
        address: String::new(),
        name: name.into(),
        advertised_service: None,
        manufacturer_data: Vec::new(),
    });
    if let Some(model) = resolved.model {
        let support = match model.support {
            protocol::SupportLevel::Verified => "verified",
            protocol::SupportLevel::Experimental => "experimental",
            protocol::SupportLevel::ScanOnly => "scanOnly",
        };
        (
            true,
            Some(model.id),
            Some(model.display_name),
            Some(support.into()),
        )
    } else {
        (false, None, None, None)
    }
}
