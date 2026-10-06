use serde::Serialize;
use tauri::{AppHandle, Manager};
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
    error: Option<String>,
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
            error: Some(error),
        }),
    }
}

#[tauri::command]
pub(crate) async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|error| format!("Updater: {error}"))?;
    let update = updater
        .check()
        .await
        .map_err(|error| format!("Check update: {error}"))?
        .ok_or_else(|| {
            "Không có bản cập nhật ký số. Mở GitHub Releases để tải thủ công.".to_string()
        })?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|error| format!("Install update: {error}"))?;
    app.restart();
    Ok(())
}

async fn github_latest_version() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("B4S-Desktop")
        .build()
        .map_err(|error| error.to_string())?;
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
    let parse = |value: &str| -> Vec<u64> {
        value
            .trim_start_matches('v')
            .split(|character: char| !character.is_ascii_digit())
            .filter_map(|part| part.parse().ok())
            .collect()
    };
    let remote = parse(remote);
    let current = parse(current);
    for index in 0..remote.len().max(current.len()) {
        let remote_part = remote.get(index).copied().unwrap_or(0);
        let current_part = current.get(index).copied().unwrap_or(0);
        if remote_part != current_part {
            return remote_part > current_part;
        }
    }
    false
}
