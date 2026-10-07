use super::error::{ApiError, ApiErrorCode};
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppInfo {
    name: String,
    version: String,
    identifier: String,
    tauri_version: String,
    os: String,
    debug: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateCheckResult {
    available: bool,
    current_version: String,
    version: Option<String>,
    body: Option<String>,
    date: Option<String>,
    error: Option<ApiError>,
}

#[tauri::command]
pub(crate) fn get_app_info(app: AppHandle) -> AppInfo {
    let pkg = app.package_info();
    AppInfo {
        name: pkg.name.clone(),
        version: pkg.version.to_string(),
        identifier: app.config().identifier.clone(),
        tauri_version: tauri::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        debug: cfg!(debug_assertions),
    }
}

/// Prefer signed Tauri updater; fall back to GitHub Releases tag compare.
#[tauri::command]
pub(crate) async fn check_for_updates(app: AppHandle) -> Result<UpdateCheckResult, String> {
    let current = app.package_info().version.to_string();
    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                return Ok(UpdateCheckResult {
                    available: true,
                    current_version: current,
                    version: Some(update.version.clone()),
                    body: update.body.clone(),
                    date: update.date.map(|date| date.to_string()),
                    error: None,
                });
            }
            Ok(None) => {
                return Ok(UpdateCheckResult {
                    available: false,
                    current_version: current,
                    version: None,
                    body: None,
                    date: None,
                    error: None,
                });
            }
            Err(error) => log::warn!("Updater check failed, trying GitHub API: {error}"),
        },
        Err(error) => log::warn!("Updater unavailable: {error}"),
    }

    match github_latest_version().await {
        Ok(remote) => {
            let available = is_remote_newer(&remote, &current);
            Ok(UpdateCheckResult {
                available,
                current_version: current,
                version: Some(remote),
                body: if available {
                    Some("Tải bản cài từ GitHub Releases (updater ký chưa sẵn sàng).".into())
                } else {
                    None
                },
                date: None,
                error: None,
            })
        }
        Err(error) => Ok(UpdateCheckResult {
            available: false,
            current_version: current,
            version: None,
            body: None,
            date: None,
            error: Some(ApiError::new(ApiErrorCode::UpdateCheckFailed, error, false)),
        }),
    }
}

#[tauri::command]
pub(crate) async fn install_update(app: AppHandle) -> Result<(), ApiError> {
    let updater = app.updater().map_err(|error| {
        ApiError::new(
            ApiErrorCode::UpdateInstallFailed,
            format!("Updater: {error}"),
            true,
        )
    })?;
    let update = updater
        .check()
        .await
        .map_err(|error| {
            ApiError::new(
                ApiErrorCode::UpdateInstallFailed,
                format!("Check update: {error}"),
                true,
            )
        })?
        .ok_or_else(|| {
            ApiError::new(
                ApiErrorCode::UpdateInstallFailed,
                "Không có bản cập nhật ký số. Mở GitHub Releases để tải thủ công.",
                false,
            )
        })?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|error| {
            ApiError::new(
                ApiErrorCode::UpdateInstallFailed,
                format!("Install update: {error}"),
                true,
            )
        })?;
    app.restart()
}

fn github_client() -> Result<reqwest::Client, reqwest::Error> {
    // reqwest 0.13 requires an explicitly installed provider with rustls-no-provider.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder().user_agent("B4S-Desktop").build()
}

async fn github_latest_version() -> Result<String, String> {
    let client = github_client().map_err(|error| error.to_string())?;
    let response = client
        .get("https://api.github.com/repos/hoan02/b4s/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("GitHub: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub releases HTTP {} (repo public + có release chưa?)",
            response.status()
        ));
    }
    let json: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    let tag = json
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or("No tag_name in release")?;
    Ok(tag.trim_start_matches('v').to_string())
}

fn is_remote_newer(remote: &str, current: &str) -> bool {
    match (
        semver::Version::parse(remote.trim_start_matches('v')),
        semver::Version::parse(current.trim_start_matches('v')),
    ) {
        (Ok(remote), Ok(current)) => remote.cmp_precedence(&current).is_gt(),
        _ => false,
    }
}

#[cfg(test)]
mod version_tests {
    use super::is_remote_newer;

    #[test]
    fn http_client_has_a_usable_tls_provider() {
        assert!(super::github_client().is_ok());
    }

    #[test]
    fn release_order_follows_semver_including_prereleases_and_metadata() {
        assert!(is_remote_newer("v0.1.10", "0.1.9"));
        assert!(!is_remote_newer("0.2.0-beta.1", "0.2.0"));
        assert!(is_remote_newer("0.2.0", "0.2.0-beta.1"));
        assert!(!is_remote_newer("0.2.0+build.2", "0.2.0+build.1"));
        assert!(!is_remote_newer("bad", "0.1.2"));
    }
}
