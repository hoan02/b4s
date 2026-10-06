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
        Self::new(ApiErrorCode::DeviceCommandFailed, message, true)
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
}
