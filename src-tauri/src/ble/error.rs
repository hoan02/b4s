use std::fmt;

/// Stable internal categories for BLE connection failures. Platform details
/// remain in `Operation`, while callers can distinguish retry and support
/// decisions without parsing display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BleError {
    ConnectionAttemptInProgress,
    DeviceUnavailable,
    UnsupportedControlTransport,
    ProtocolUnconfigured,
    SessionCancelled,
    Operation(String),
}

impl BleError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionAttemptInProgress | Self::SessionCancelled | Self::Operation(_)
        )
    }
}

impl fmt::Display for BleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionAttemptInProgress => {
                formatter.write_str("Another connection attempt is active")
            }
            Self::DeviceUnavailable => formatter.write_str("Device is no longer in the scan list"),
            Self::UnsupportedControlTransport => formatter.write_str(
                "This model has no reviewed BLE control transport; capture/transport verification is required",
            ),
            Self::ProtocolUnconfigured => formatter.write_str(
                "This model is recognized only; its Bluetooth control protocol is not configured",
            ),
            Self::SessionCancelled => formatter.write_str("Connection attempt was cancelled"),
            Self::Operation(message) => formatter.write_str(message),
        }
    }
}

impl From<String> for BleError {
    fn from(message: String) -> Self {
        Self::Operation(message)
    }
}

/// Stable internal categories for BLE scan start/stop failures. Platform
/// details stay in `Operation`; adapter/BT-power categories need a user action
/// before a repeat scan can succeed, so they are not auto-retryable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    AdapterUnavailable,
    BluetoothDisabled,
    Operation(String),
}

impl ScanError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Operation(_))
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterUnavailable => formatter.write_str("No Bluetooth adapter is available"),
            Self::BluetoothDisabled => formatter.write_str("Bluetooth đang tắt trên thiết bị này"),
            Self::Operation(message) => formatter.write_str(message),
        }
    }
}

impl From<String> for ScanError {
    fn from(message: String) -> Self {
        Self::Operation(message)
    }
}

#[cfg(test)]
mod tests {
    use super::{BleError, ScanError};

    #[test]
    fn support_and_session_failures_have_distinct_retry_policies() {
        assert!(!BleError::UnsupportedControlTransport.is_retryable());
        assert!(!BleError::ProtocolUnconfigured.is_retryable());
        assert!(!BleError::DeviceUnavailable.is_retryable());
        assert!(BleError::SessionCancelled.is_retryable());
        assert!(BleError::Operation("adapter busy".into()).is_retryable());
    }

    #[test]
    fn adapter_and_power_failures_require_a_user_action_before_retry() {
        assert!(!ScanError::AdapterUnavailable.is_retryable());
        assert!(!ScanError::BluetoothDisabled.is_retryable());
        assert!(ScanError::Operation("start_scan: busy".into()).is_retryable());
        assert_eq!(
            ScanError::from("start_scan: busy".to_owned()),
            ScanError::Operation("start_scan: busy".into())
        );
    }
}
