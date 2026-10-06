//! Own the adapter event stream once; scan timers do not own connection events.

use super::*;

fn event_is_current_connection(
    current_session: crate::device::session::SessionToken,
    event_session: crate::device::session::SessionToken,
    active_id: Option<&str>,
    event_id: &str,
) -> bool {
    current_session == event_session && active_id == Some(event_id)
}

pub(super) async fn ensure_central_listener(
    app: AppHandle,
    adapter: Adapter,
) -> Result<(), String> {
    let mut state = BLE.lock().await;
    if state
        .central_task
        .as_ref()
        .is_some_and(|task| !task.is_finished())
    {
        return Ok(());
    }
    // Subscribe before starting scan, so initial discovery events are not lost.
    let events = adapter
        .events()
        .await
        .map_err(|error| format!("events: {error}"))?;
    state.central_task = Some(tokio::spawn(async move {
        listen_central_events(app, adapter, events).await;
    }));
    Ok(())
}

/// Stop and join the app-scoped adapter event listener during bounded Quit cleanup.
pub(super) async fn stop_central_listener() {
    let task = BLE.lock().await.central_task.take();
    if let Some(task) = task {
        task.abort();
        let _ = task.await;
    }
}

async fn listen_central_events(
    app: AppHandle,
    adapter: Adapter,
    mut events: std::pin::Pin<Box<dyn futures::Stream<Item = CentralEvent> + Send>>,
) {
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                let scan_generation = {
                    let state = BLE.lock().await;
                    if !state.scanning {
                        continue;
                    }
                    state.scan_generation
                };
                if let Ok(p) = adapter.peripheral(&id).await {
                    super::scanning::process_peripheral(&app, p, &id, scan_generation).await;
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                let id_str = id_to_string(&id);
                let (was_active, token) = {
                    let state = BLE.lock().await;
                    (
                        state.connected_id.as_ref() == Some(&id_str),
                        state.session.token(),
                    )
                };
                if was_active {
                    let peripheral_still_connected = match adapter.peripheral(&id).await {
                        Ok(peripheral) => peripheral.is_connected().await.unwrap_or(false),
                        Err(_) => false,
                    };
                    if peripheral_still_connected {
                        continue;
                    }
                }

                let (still_active, session_tasks) = {
                    let mut state = BLE.lock().await;
                    let still_active = was_active
                        && event_is_current_connection(
                            state.session.token(),
                            token,
                            state.connected_id.as_deref(),
                            &id_str,
                        );
                    let session_tasks = if still_active {
                        state.connected_id = None;
                        state.battery = BatteryState::default();
                        state.reset_link()
                    } else {
                        Vec::new()
                    };
                    if still_active || state.connected_id.as_ref() != Some(&id_str) {
                        if let Some(d) = state.devices.get_mut(&id_str) {
                            d.connected = false;
                        }
                    }
                    (still_active, session_tasks)
                };
                super::runtime::join_session_tasks(session_tasks).await;
                if still_active {
                    emit_connection_state(&app).await;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::session::SessionEpoch;

    #[test]
    fn delayed_disconnect_cannot_clear_a_reconnected_session() {
        let mut sessions = SessionEpoch::default();
        let old_session = sessions.token();
        sessions.invalidate();
        let current_session = sessions.token();

        assert!(!event_is_current_connection(
            current_session,
            old_session,
            Some("selected-entry"),
            "selected-entry",
        ));
        assert!(!event_is_current_connection(
            current_session,
            current_session,
            Some("new-entry"),
            "selected-entry",
        ));
        assert!(event_is_current_connection(
            current_session,
            current_session,
            Some("selected-entry"),
            "selected-entry",
        ));
    }
}
