use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ApiErrorCode {
    InvalidRequest,
    DeviceUnavailable,
    ScanFailed,
    ConnectionFailed,
    DisconnectFailed,
    BatteryReadFailed,
    DeviceCommandFailed,
    PreferenceFailed,
    UpdateCheckFailed,
    UpdateInstallFailed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiError {
    contract_version: u16,
    code: ApiErrorCode,
    message: String,
    retryable: bool,
}

impl ApiError {
    pub(crate) fn new(code: ApiErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            contract_version: 1,
            code,
            message: message.into(),
            retryable,
        }
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        // A failed command may have reached the device before readback failed;
        // callers must inspect state before deciding whether to issue it again.
        Self::new(ApiErrorCode::DeviceCommandFailed, message, false)
    }
}

impl From<crate::ble::BleError> for ApiError {
    fn from(error: crate::ble::BleError) -> Self {
        let code = match &error {
            crate::ble::BleError::DeviceUnavailable
            | crate::ble::BleError::UnsupportedControlTransport
            | crate::ble::BleError::ProtocolUnconfigured => ApiErrorCode::DeviceUnavailable,
            _ => ApiErrorCode::ConnectionFailed,
        };
        let retryable = error.is_retryable();
        Self::new(code, error.to_string(), retryable)
    }
}

impl From<crate::ble::ScanError> for ApiError {
    fn from(error: crate::ble::ScanError) -> Self {
        Self::new(
            ApiErrorCode::ScanFailed,
            error.to_string(),
            error.is_retryable(),
        )
    }
}

impl From<crate::ble::CommandError> for ApiError {
    fn from(error: crate::ble::CommandError) -> Self {
        let code = match &error {
            crate::ble::CommandError::NotConnected
            | crate::ble::CommandError::UnsupportedFeature(_)
            | crate::ble::CommandError::SessionCancelled => ApiErrorCode::DeviceUnavailable,
            _ => ApiErrorCode::DeviceCommandFailed,
        };
        Self::new(code, error.to_string(), error.is_retryable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_errors_are_versioned_and_machine_readable() {
        let error = ApiError::new(ApiErrorCode::ConnectionFailed, "adapter unavailable", true);
        let value = serde_json::to_value(error).unwrap();
        assert_eq!(value["contractVersion"], 1);
        assert_eq!(value["code"], "connectionFailed");
        assert_eq!(value["message"], "adapter unavailable");
        assert_eq!(value["retryable"], true);
    }

    #[test]
    fn uncertain_device_command_failures_do_not_claim_blind_retry_is_safe() {
        let value = serde_json::to_value(ApiError::from("readback timed out".to_owned())).unwrap();
        assert_eq!(value["code"], "deviceCommandFailed");
        assert_eq!(value["retryable"], false);
    }

    #[test]
    fn unsupported_ble_profiles_map_to_non_retryable_device_unavailable() {
        let value = serde_json::to_value(ApiError::from(
            crate::ble::BleError::UnsupportedControlTransport,
        ))
        .unwrap();
        assert_eq!(value["code"], "deviceUnavailable");
        assert_eq!(value["retryable"], false);
    }

    #[test]
    fn scan_failures_expose_power_state_and_retry_policy() {
        let disabled =
            serde_json::to_value(ApiError::from(crate::ble::ScanError::BluetoothDisabled)).unwrap();
        assert_eq!(disabled["code"], "scanFailed");
        assert_eq!(disabled["retryable"], false);

        let busy = serde_json::to_value(ApiError::from(crate::ble::ScanError::Operation(
            "start_scan: busy".into(),
        )))
        .unwrap();
        assert_eq!(busy["code"], "scanFailed");
        assert_eq!(busy["retryable"], true);
    }

    #[test]
    fn command_failures_expose_connection_state_and_retry_policy() {
        let disconnected =
            serde_json::to_value(ApiError::from(crate::ble::CommandError::NotConnected)).unwrap();
        assert_eq!(disconnected["code"], "deviceUnavailable");
        assert_eq!(disconnected["retryable"], false);

        let denied = serde_json::to_value(ApiError::from(
            crate::ble::CommandError::UnsupportedFeature("Eq is not supported".into()),
        ))
        .unwrap();
        assert_eq!(denied["code"], "deviceUnavailable");
        assert_eq!(denied["retryable"], false);

        let queued =
            serde_json::to_value(ApiError::from(crate::ble::CommandError::QueueFull)).unwrap();
        assert_eq!(queued["code"], "deviceCommandFailed");
        assert_eq!(queued["retryable"], true);

        let uncertain = serde_json::to_value(ApiError::from(crate::ble::CommandError::Operation(
            "readback timed out".into(),
        )))
        .unwrap();
        assert_eq!(uncertain["code"], "deviceCommandFailed");
        assert_eq!(uncertain["retryable"], false);
    }
}
