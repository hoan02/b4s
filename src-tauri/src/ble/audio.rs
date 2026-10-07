//! Windows audio-to-control association. Names alone never establish identity.
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTarget {
    pub endpoint_id: String,
    pub name: String,
    pub candidates: Vec<AudioCandidate>,
}

#[derive(Clone, Serialize)]
pub struct AudioCandidate {
    pub address: String,
    pub name: String,
}

#[cfg(any(target_os = "windows", test))]
fn same_container<T: Eq>(output: Option<T>, entry: Option<T>) -> bool {
    output.is_some() && output == entry
}

#[cfg(not(target_os = "windows"))]
pub async fn current_target() -> Result<Option<AudioTarget>, String> {
    Ok(None)
}

#[cfg(target_os = "windows")]
pub async fn current_target() -> Result<Option<AudioTarget>, String> {
    tokio::time::timeout(std::time::Duration::from_secs(5), windows_target())
        .await
        .map_err(|_| "Windows audio device lookup timed out".to_string())?
        .map_err(|error| format!("Windows audio device lookup: {error}"))
}

#[cfg(target_os = "windows")]
async fn windows_target() -> windows::core::Result<Option<AudioTarget>> {
    use windows::{
        core::{Interface, GUID, HSTRING},
        Devices::{Bluetooth::BluetoothLEDevice, Enumeration::DeviceInformation},
        Foundation::IPropertyValue,
        Media::Devices::{AudioDeviceRole, MediaDevice},
    };
    use windows_collections::IIterable;

    fn container(info: &DeviceInformation) -> Option<GUID> {
        let properties = info.Properties().ok()?;
        for key in [
            "System.Devices.ContainerId",
            "System.Devices.Aep.ContainerId",
        ] {
            let id = properties
                .Lookup(&HSTRING::from(key))
                .ok()
                .and_then(|value| value.cast::<IPropertyValue>().ok())
                .and_then(|value| value.GetGuid().ok());
            if let Some(id) = id.filter(|id| *id != GUID::zeroed()) {
                return Some(id);
            }
        }
        None
    }

    let endpoint = MediaDevice::GetDefaultAudioRenderId(AudioDeviceRole::Default)?;
    if endpoint.is_empty() {
        return Ok(None);
    }
    let operation = {
        let properties: IIterable<HSTRING> =
            vec![HSTRING::from("System.Devices.ContainerId")].into();
        DeviceInformation::CreateFromIdAsyncAdditionalProperties(&endpoint, &properties)?
    };
    let output = operation.await?;
    let Some(output_container) = container(&output) else {
        return Ok(None);
    };
    // Enumerate OS-known LE interfaces; this does not start a radio advertisement scan.
    let selector = BluetoothLEDevice::GetDeviceSelector()?;
    let operation = {
        let properties: IIterable<HSTRING> = vec![
            HSTRING::from("System.Devices.ContainerId"),
            HSTRING::from("System.Devices.Aep.ContainerId"),
        ]
        .into();
        DeviceInformation::FindAllAsyncAqsFilterAndAdditionalProperties(&selector, &properties)?
    };
    let entries = operation.await?;
    let mut candidates = Vec::new();
    for index in 0..entries.Size()? {
        let entry = entries.GetAt(index)?;
        if !same_container(Some(output_container), container(&entry)) {
            continue;
        }
        let Ok(device) = BluetoothLEDevice::FromIdAsync(&entry.Id()?)?.await else {
            continue;
        };
        let address = device.BluetoothAddress()?;
        let name = device.Name()?.to_string();
        device.Close()?;
        if address == 0 || address > 0xFFFF_FFFF_FFFF {
            continue;
        }
        let hex = format!("{address:012X}");
        let address = (0..6)
            .map(|i| &hex[i * 2..i * 2 + 2])
            .collect::<Vec<_>>()
            .join(":");
        if !candidates
            .iter()
            .any(|candidate: &AudioCandidate| candidate.address == address)
        {
            candidates.push(AudioCandidate { address, name });
        }
    }
    candidates.retain(|candidate| eligible_device(candidate, String::new()).is_some());
    if candidates.is_empty() {
        return Ok(None);
    }
    // Do not publish a stale target when Windows switches output during enumeration.
    if MediaDevice::GetDefaultAudioRenderId(AudioDeviceRole::Default)? != endpoint {
        return Ok(None);
    }
    Ok(Some(AudioTarget {
        endpoint_id: endpoint.to_string(),
        name: output.Name()?.to_string(),
        candidates,
    }))
}

fn eligible_device(candidate: &AudioCandidate, id: String) -> Option<super::BleDevice> {
    let device = super::scanning::device_from_identity(
        id,
        -100,
        Vec::new(),
        crate::device::DeviceIdentity {
            address: candidate.address.clone(),
            name: candidate.name.clone(),
            advertised_service: None,
            manufacturer_data: Vec::new(),
        },
    );
    (device.headphone_candidate
        && device
            .device_profile
            .connection
            .as_ref()
            .is_some_and(|connection| {
                connection.transport == crate::catalog::ControlTransport::BleGatt
            })
        && device.device_profile.protocol != crate::protocol::ProtocolFamily::Unknown)
        .then_some(device)
}

fn unique_candidate(target: &AudioTarget) -> Result<&AudioCandidate, String> {
    let mut candidates = target
        .candidates
        .iter()
        .filter(|candidate| eligible_device(candidate, String::new()).is_some());
    let candidate = candidates
        .next()
        .ok_or("The active audio device has no supported BLE control profile; scan manually")?;
    if candidates.next().is_some() {
        return Err("Windows reports several BLE control entries for this output; scan and select one explicitly".into());
    }
    Ok(candidate)
}

/// Recheck at command admission as well as after enumeration.
pub fn ensure_current_output(endpoint_id: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let current = windows::Media::Devices::MediaDevice::GetDefaultAudioRenderId(
            windows::Media::Devices::AudioDeviceRole::Default,
        )
        .map_err(|error| error.to_string())?;
        if current.to_string() == endpoint_id {
            return Ok(());
        }
        Err("The audio output changed; refresh and try again".into())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = endpoint_id;
        Err("Audio quick connect is available on Windows only".into())
    }
}

pub async fn prepare_target(
    app: tauri::AppHandle,
    endpoint_id: &str,
) -> Result<super::BleDevice, String> {
    let target = current_target()
        .await?
        .filter(|target| target.endpoint_id == endpoint_id)
        .ok_or(
            "The audio output changed or its BLE identity is unavailable; refresh and try again",
        )?;
    let candidate = unique_candidate(&target)?;
    #[cfg(target_os = "windows")]
    {
        use btleplug::api::{Central, Peripheral as _};
        super::init_adapter().await?;
        let _operation = super::adapter::SCAN_OPERATION.lock().await;
        let adapter = super::adapter::current()
            .await
            .ok_or("Bluetooth adapter unavailable")?;
        super::discovery::ensure_central_listener(app, adapter.clone()).await?;
        let address: btleplug::api::BDAddr = candidate
            .address
            .parse()
            .map_err(|_| "Invalid Windows Bluetooth address")?;
        let peripheral = adapter
            .add_peripheral(&address.into())
            .await
            .map_err(|error| error.to_string())?;
        let device = eligible_device(candidate, super::id_to_string(&peripheral.id()))
            .ok_or("Unsupported control profile")?;
        ensure_current_output(endpoint_id)?;
        let mut state = super::BLE.lock().await;
        if state.scanning {
            return Err("Stop scanning before using audio quick connect".into());
        }
        state.devices.insert(device.id.clone(), device.clone());
        state.peripherals.insert(device.id.clone(), peripheral);
        Ok(device)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (candidate, app);
        Err("Audio quick connect is available on Windows only".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate(address: &str, name: &str) -> AudioCandidate {
        AudioCandidate {
            address: address.into(),
            name: name.into(),
        }
    }
    #[test]
    fn missing_and_different_container_ids_never_match() {
        assert!(same_container(Some(1), Some(1)));
        assert!(!same_container(Some(1), Some(2)));
        assert!(!same_container(Some(1), None));
        assert!(!same_container(None, Some(1)));
        assert!(!same_container::<u8>(None, None));
    }
    #[test]
    fn only_one_reviewed_control_entry_can_be_selected() {
        let mut target = AudioTarget {
            endpoint_id: "endpoint".into(),
            name: "audio".into(),
            candidates: vec![candidate("AA:BB:CC:DD:EE:FF", "Baseus Bass BP1 Pro")],
        };
        assert!(unique_candidate(&target).is_ok());
        target
            .candidates
            .push(candidate("AA:BB:CC:DD:EE:FE", "Baseus Bass BP1 Pro"));
        assert!(unique_candidate(&target).is_err());
    }
    #[test]
    fn unknown_and_scan_only_models_cannot_open_control() {
        assert!(eligible_device(
            &candidate("AA:BB:CC:DD:EE:FF", "Unknown earbuds"),
            "id".into()
        )
        .is_none());
        assert!(eligible_device(
            &candidate("AA:BB:CC:DD:EE:FF", "Baseus Bowie MA10"),
            "id".into()
        )
        .is_none());
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    #[ignore = "Requires a live Windows audio/device enumeration environment"]
    async fn windows_audio_lookup_smoke() {
        let target = current_target()
            .await
            .expect("Windows audio enumeration should complete");
        println!(
            "Supported audio-to-BLE association available: {}",
            target.is_some()
        );
    }
}
