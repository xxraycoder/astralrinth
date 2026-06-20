///
/// This file is modified by AstralRinth
///
use crate::api::astralrinth::update;
use crate::event::emit::emit_info;
use crate::{Result, State};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;
use tokio::{fs, io};

const PACKAGE_JSON_CONTENT: &str =
    // include_str!("../../../../../apps/app-frontend/package.json");
    include_str!("../../../../../apps/app/tauri.conf.json");

/// Deserialize the content of package.json into a Launcher struct
pub fn read_package_json() -> io::Result<Launcher> {
    let launcher: Launcher = serde_json::from_str(PACKAGE_JSON_CONTENT)?;
    Ok(launcher)
}

#[derive(Serialize, Deserialize)]
pub struct Launcher {
    pub version: String,
}

/// Fetches or updates the Ely.by AuthLib Injector library.
pub async fn get_elyby_injector_library() -> Result<PathBuf> {
    tracing::info!("[AR] • Initializing Ely.by AuthLib Injector...");
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();

    validate_library_dir(&libraries_dir, "authlib_injector/").await?;
    let injector_dir = libraries_dir.join("astralrinth/authlib_injector/");
    fs::create_dir_all(&injector_dir).await?;

    let mut local_injectors = Vec::new();
    if let Ok(mut entries) = fs::read_dir(&injector_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let (Some(name), Ok(meta)) = (
                path.file_name().and_then(|s| s.to_str()),
                entry.metadata().await,
            ) {
                if name.starts_with("authlib-injector") {
                    local_injectors.push((
                        path,
                        meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    ));
                }
            }
        }
    }
    local_injectors.sort_by(|a, b| b.1.cmp(&a.1)); // newest first

    if !local_injectors.is_empty() {
        tracing::info!("[AR] • Local versions:");
        for (path, mtime) in &local_injectors {
            tracing::info!("  • {:?} ({:?})", path.file_name().unwrap(), mtime);
        }
    }

    let latest_local = local_injectors.first().cloned();
    // let latest_local_name = latest_local.as_ref().map(|p| p.0.file_name().unwrap().to_string_lossy().to_string());

    // Remote (fallback to empty strings)
    let (remote_name, remote_url) =
        match extract_metadata_from_elyby_file("authlib-injector").await {
            Ok(data) => {
                tracing::info!("[AR] • Remote: {} ({})", data.0, data.1);
                data
            }
            Err(e) => {
                tracing::warn!("[AR] • Remote failed: {}, using local", e);
                ("".to_string(), "".to_string())
            }
        };

    let remote_path = if !remote_name.is_empty() {
        Some(injector_dir.join(&remote_name))
    } else {
        None
    };

    if let Some(local_path) = &latest_local {
        let local_name = local_path.0.file_name().unwrap().to_string_lossy();
        if let Some(rp) = &remote_path {
            let remote_name = rp.file_name().unwrap().to_string_lossy();
            if local_name == remote_name {
                tracing::info!("[AR] • Versions match: {}", local_name);
                return Ok(local_path.0.clone());
            }
        } else {
            tracing::info!(
                "[AR] • No remote info, using local: {}",
                local_name
            );
            let _ = emit_info(&format!(
                "[AR] No remote info, using local: {}",
                local_name
            ))
            .await;
            return Ok(local_path.0.clone());
        }
    }

    let Some(rp) = remote_path else {
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: "No local injector & remote unavailable".to_string(),
        }
        .as_error());
    };

    let fname = rp.file_name().unwrap().to_string_lossy();
    tracing::info!("[AR] • Downloading: {}", fname);
    let _ = emit_info(&format!("[AR] Downloading: {}", fname)).await;

    let bytes = fetch_bytes_from_url(&remote_url).await?;
    let rel_path = rp
        .strip_prefix(&libraries_dir)?
        .to_string_lossy()
        .into_owned();
    write_file_to_libraries(&rel_path, &bytes).await?;

    tracing::info!("[AR] • Saved: {}", rp.display());
    let _ = emit_info(&format!("[AR] Saved: {}", rp.display())).await;
    Ok(rp.to_path_buf())
}

/// Parses the ElyIntegration release JSON and returns the download URL for the given AuthLib version.
async fn extract_metadata_from_elyby_file(
    file_name: &str,
) -> Result<(String, String)> {
    const URL: &str = "https://git.astralium.su/api/v1/repos/didirus/ElyIntegration/releases/latest";

    let response = reqwest::get(URL).await.map_err(|e| {
        tracing::error!(
            "[AR] • Failed to fetch ElyIntegration release JSON: {:?}",
            e
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Failed to fetch ElyIntegration release JSON: {}",
                e
            ),
        }
        .as_error()
    })?;

    let json: serde_json::Value = response.json().await.map_err(|e| {
        tracing::error!("[AR] • Failed to parse ElyIntegration JSON: {:?}", e);
        crate::ErrorKind::ParseError {
            reason: format!("Failed to parse ElyIntegration JSON: {}", e),
        }
        .as_error()
    })?;

    let assets =
        json.get("assets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::ErrorKind::ParseError {
                    reason: "Missing 'assets' array".into(),
                }
                .as_error()
            })?;

    let asset = assets
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.contains(file_name))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: format!(
                    "No matching asset for {} in ElyIntegration JSON response.",
                    file_name
                ),
            }
            .as_error()
        })?;

    let download_url = asset
        .get("browser_download_url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'browser_download_url'".into(),
            }
            .as_error()
        })?
        .to_string();

    let asset_name = asset
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'name'".into(),
            }
            .as_error()
        })?
        .to_string();

    Ok((asset_name, download_url))
}

/// Initialize the update launcher.
pub async fn init_update_launcher(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
    auto_update_supported: bool,
) -> Result<()> {
    tracing::info!("[AR] • Initialize downloading from • {:?}", download_url);
    tracing::info!("[AR] • Save local file name • {:?}", local_filename);
    tracing::info!("[AR] • OS type • {}", os_type);
    tracing::info!("[AR] • Auto update supported • {}", auto_update_supported);

    if let Err(e) = update::get_resource(
        download_url,
        local_filename,
        os_type,
        auto_update_supported,
    )
    .await
    {
        eprintln!(
            "[AR] • An error occurred! Failed to download the file: {}",
            e
        );
    } else {
        println!("[AR] • Code finishes without errors.");
        process::exit(0)
    }
    Ok(())
}

/// Validating the `astralrinth/{target_directory}/` directory exists inside the libraries/astralrinth directory.
async fn validate_library_dir(
    libraries_dir: &PathBuf,
    validation_directory: &str,
) -> Result<()> {
    let astralrinth_path =
        libraries_dir.join(format!("astralrinth/{}", validation_directory));
    if !astralrinth_path.exists() {
        tokio::fs::create_dir_all(&astralrinth_path)
            .await
            .map_err(|e| {
                tracing::error!(
                    "[AR] • Failed to create {} directory: {:?}",
                    astralrinth_path.display(),
                    e
                );
                crate::ErrorKind::IOErrorOccurred {
                    error: format!(
                        "Failed to create {} directory: {}",
                        astralrinth_path.display(),
                        e
                    ),
                }
                .as_error()
            })?;
        tracing::info!(
            "[AR] • Created missing {} directory",
            astralrinth_path.display()
        );
    }
    Ok(())
}

/// Saves the downloaded bytes to the `libraries` directory using the given relative path.
async fn write_file_to_libraries(
    relative_path: &str,
    bytes: &bytes::Bytes,
) -> Result<()> {
    let state = State::get().await?;
    let output_path = state.directories.libraries_dir().join(relative_path);

    fs::write(&output_path, bytes).await.map_err(|e| {
        tracing::error!("[AR] • Failed to save file: {:?}", e);
        crate::ErrorKind::IOErrorOccurred {
            error: format!("Failed to save file: {e}"),
        }
        .as_error()
    })
}

/// Downloads bytes from the provided URL with a 15 second timeout.
async fn fetch_bytes_from_url(url: &str) -> Result<bytes::Bytes> {
    // Create client instance with request timeout.
    let client = reqwest::Client::new();
    const TIMEOUT_SECONDS: u64 = 5;

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(TIMEOUT_SECONDS),
        client.get(url).send(),
    )
    .await
    .map_err(|_| {
        tracing::error!(
            "[AR] • Download timed out after {} seconds",
            TIMEOUT_SECONDS
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Download timed out after {TIMEOUT_SECONDS} seconds"
            )
            .to_string(),
        }
        .as_error()
    })?
    .map_err(|e| {
        tracing::error!("[AR] • Request error: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Request error: {e}"),
        }
        .as_error()
    })?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        tracing::error!("[AR] • Failed to download file: HTTP {}", status);
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to download file: HTTP {status}"),
        }
        .as_error());
    }

    response.bytes().await.map_err(|e| {
        tracing::error!("[AR] • Failed to read response bytes: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to read response bytes: {e}"),
        }
        .as_error()
    })
}
