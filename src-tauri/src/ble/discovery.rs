//! Own the adapter event stream once; scan timers do not own connection events.

use super::*;

pub(super) async fn ensure_central_listener(app: AppHandle, adapter: Adapter) -> Result<(), String> {
    let mut state = BLE.lock().await;
    if state.central_task.as_ref().is_some_and(|task| !task.is_finished()) {
        return Ok(());
    }
    // Subscribe before starting scan, so initial discovery events are not lost.
    let events = adapter.events().await.map_err(|error| format!("events: {error}"))?;
    state.central_task = Some(tokio::spawn(async move {
        listen_central_events(app, adapter, events).await;
    }));
    Ok(())
}

async fn listen_central_events(
    app: AppHandle,
    adapter: Adapter,
    mut events: std::pin::Pin<Box<dyn futures::Stream<Item = CentralEvent> + Send>>,
) {
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                if !BLE.lock().await.scanning { continue; }
                if let Ok(p) = adapter.peripheral(&id).await {
                    process_peripheral(&app, p, &id).await;
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                let id_str = id_to_string(&id);
                let mut state = BLE.lock().await;
                let was_active = state.connected_id.as_ref() == Some(&id_str);
                if was_active {
                    state.connected_id = None;
                    state.battery = BatteryState::default();
                    state.reset_link();
                }
                if let Some(d) = state.devices.get_mut(&id_str) {
                    d.connected = false;
                }
                drop(state);
                if was_active {
                    emit_connection_state(&app).await;
                    let _ = app.emit("ble://disconnected", &id_str);
                }
            }
            _ => {}
        }
    }
}

