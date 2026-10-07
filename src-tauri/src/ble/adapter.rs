//! Own the platform adapter and serialize its app-scoped operations.

use super::*;
use std::sync::LazyLock;

pub(super) static INITIALIZATION: Mutex<()> = Mutex::const_new(());
pub(super) static SCAN_OPERATION: Mutex<()> = Mutex::const_new(());
pub(super) static CENTRAL_LISTENER_OPERATION: Mutex<()> = Mutex::const_new(());

#[derive(Default)]
struct AdapterState {
    adapter: Option<Adapter>,
    central_task: Option<tokio::task::JoinHandle<()>>,
}

static STATE: LazyLock<Mutex<AdapterState>> = LazyLock::new(|| Mutex::new(AdapterState::default()));

pub(super) async fn initialize() -> Result<Adapter, String> {
    let _initialization = INITIALIZATION.lock().await;
    if let Some(adapter) = STATE.lock().await.adapter.clone() {
        return Ok(adapter);
    }

    let manager = Manager::new()
        .await
        .map_err(|error| format!("BLE manager: {error}"))?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|error| format!("List adapters: {error}"))?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or_else(|| "No Bluetooth adapter found".to_string())?;

    let mut state = STATE.lock().await;
    if state.adapter.is_none() {
        log::info!("BLE adapter ready");
        state.adapter = Some(adapter);
    }
    state
        .adapter
        .clone()
        .ok_or_else(|| "Bluetooth adapter initialization failed".to_string())
}

pub(super) async fn current() -> Option<Adapter> {
    STATE.lock().await.adapter.clone()
}

pub(super) async fn has_central_listener() -> bool {
    STATE
        .lock()
        .await
        .central_task
        .as_ref()
        .is_some_and(|task| !task.is_finished())
}

pub(super) async fn install_central_listener(task: tokio::task::JoinHandle<()>) {
    let mut state = STATE.lock().await;
    state.central_task = Some(task);
}

pub(super) async fn take_central_listener() -> Option<tokio::task::JoinHandle<()>> {
    STATE.lock().await.central_task.take()
}
