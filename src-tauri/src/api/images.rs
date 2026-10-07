//! Bounded disk cache for catalog product art, independent of BLE commands.
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
use tauri::Manager;

const MAX_IMAGE: u64 = 8 * 1024 * 1024;
const MAX_CACHE: u64 = 64 * 1024 * 1024;
static DOWNLOADS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
static CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("image HTTP client")
});

fn image_extension(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if data.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("jpg")
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

fn catalog_url(url: &str) -> bool {
    url.starts_with("https://")
        && (crate::catalog::all_profiles()
            .iter()
            .any(|profile| profile.image.as_deref() == Some(url))
            || crate::catalog::public::headphone_models().any(|model| {
                model.variants.iter().any(|variant| {
                    variant.image_url.as_deref() == Some(url)
                        || variant.large_image_url.as_deref() == Some(url)
                        || variant
                            .colors
                            .iter()
                            .any(|color| color.image_url.as_deref() == Some(url))
                })
            }))
}

fn prune(directory: &Path, retained: &Path) -> std::io::Result<()> {
    let mut entries = Vec::new();
    let mut total = 0;
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("png" | "jpg" | "webp")
        ) {
            continue;
        }
        let metadata = entry.metadata()?;
        total += metadata.len();
        entries.push((metadata.modified()?, metadata.len(), path));
    }
    entries.sort_by_key(|entry| entry.0);
    for (_, size, path) in entries {
        if total <= MAX_CACHE {
            break;
        }
        if path != retained && std::fs::remove_file(path).is_ok() {
            total -= size;
        }
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn cache_product_image(
    app: tauri::AppHandle,
    url: String,
) -> Result<String, String> {
    if !catalog_url(&url) {
        return Err("Image URL is not in the product catalog".into());
    }
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("product-images");
    cache_image(&directory, &url).await
}

async fn cache_image(directory: &Path, url: &str) -> Result<String, String> {
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let key = format!("{:x}", Sha256::digest(url.as_bytes()));
    for extension in ["png", "jpg", "webp"] {
        let path = directory.join(format!("{key}.{extension}"));
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if metadata.len() <= MAX_IMAGE {
                if let Ok(bytes) = tokio::fs::read(&path).await {
                    if image_extension(&bytes) == Some(extension) {
                        return Ok(path.to_string_lossy().into_owned());
                    }
                }
            }
            let _ = tokio::fs::remove_file(path).await;
        }
    }
    let _permit = DOWNLOADS
        .acquire()
        .await
        .map_err(|error| error.to_string())?;
    let mut response = CLIENT
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_IMAGE)
    {
        return Err("Product image is too large".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if bytes.len() + chunk.len() > MAX_IMAGE as usize {
            return Err("Product image is too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let extension = image_extension(&bytes).ok_or("Unsupported product image format")?;
    let path = directory.join(format!("{key}.{extension}"));
    let temporary = directory.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    if let Err(error) = tokio::fs::write(&temporary, bytes).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error.to_string());
    }
    if let Err(error) = tokio::fs::rename(&temporary, &path).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        if !path.exists() {
            return Err(error.to_string());
        }
    }
    let prune_directory = directory.to_path_buf();
    let retained = path.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || prune(&prune_directory, &retained)).await;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_only_accepts_catalog_art_and_supported_image_signatures() {
        assert!(!catalog_url("https://example.com/arbitrary.png"));
        let url = crate::catalog::public::headphone_models()
            .next()
            .unwrap()
            .product_image()
            .unwrap();
        assert!(catalog_url(&url));
        assert_eq!(image_extension(b"<html>error</html>"), None);
        assert_eq!(image_extension(b"\x89PNG\r\n\x1a\nimage"), Some("png"));
        assert_eq!(image_extension(&[0xff, 0xd8, 0xff]), Some("jpg"));
        assert_eq!(image_extension(b"RIFF0000WEBP"), Some("webp"));
    }

    #[tokio::test]
    async fn existing_disk_image_is_reused_without_network_and_corruption_is_removed() {
        let directory = std::env::temp_dir().join(format!("b4s-images-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        // An unreachable URL proves a valid cached hit does not perform HTTP.
        let url = "http://127.0.0.1:1/not-reachable";
        let path = directory.join(format!("{:x}.png", Sha256::digest(url.as_bytes())));
        std::fs::write(&path, b"\x89PNG\r\n\x1a\ncached").unwrap();
        assert_eq!(
            cache_image(&directory, url).await.unwrap(),
            path.to_string_lossy()
        );
        std::fs::write(&path, b"broken").unwrap();
        assert!(cache_image(&directory, url).await.is_err());
        assert!(!path.exists());
        std::fs::remove_dir(&directory).unwrap();
    }

    #[test]
    fn disk_budget_evicts_old_art_and_keeps_the_new_image() {
        let directory = std::env::temp_dir().join(format!("b4s-images-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let old = directory.join("old.png");
        let retained = directory.join("new.png");
        std::fs::File::create(&old)
            .unwrap()
            .set_len(MAX_CACHE)
            .unwrap();
        std::fs::write(&retained, b"image").unwrap();
        prune(&directory, &retained).unwrap();
        assert!(!old.exists());
        assert!(retained.exists());
        std::fs::remove_file(retained).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
