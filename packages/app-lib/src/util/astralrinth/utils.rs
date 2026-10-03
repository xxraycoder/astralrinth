///
/// This file is modified by AstralRinth
///
use crate::api::astralrinth::update;
use crate::event::emit::emit_info;
use crate::models::astralrinth::authentication::ExternalAuthLibrary;
use crate::{Result, State};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process;
use tokio::{fs, io};

const PACKAGE_JSON_CONTENT: &str =
    include_str!("../../../../../apps/app-frontend/package.json");

/// Deserialize the content of package.json into a Launcher struct
pub fn read_package_json() -> io::Result<Launcher> {
    let launcher: Launcher = serde_json::from_str(PACKAGE_JSON_CONTENT)?;
    Ok(launcher)
}

#[derive(Serialize, Deserialize)]
pub struct Launcher {
    pub version: String,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryRelease {
	assets: ExternalAuthLibraryAssets,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryAssets {
	#[serde(rename = "authlib-injector")]
	authlib_injector: Vec<ExternalAuthLibraryAsset>,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryAsset {
	name: String,
	browser_download_url: Option<String>,
}

impl ExternalAuthLibraryAsset {
	fn browser_download_url(&self) -> Result<&str> {
		self.browser_download_url
			.as_deref()
			.filter(|url| !url.trim().is_empty())
			.ok_or_else(|| {
				crate::ErrorKind::ParseError {
					reason: format!(
											"Library asset has no download URL: {}",
											self.name,
										),
				}
				.as_error()
			})
	}
}

/// Resolves the library selected in SQLite and verifies its local file.
pub async fn get_authlib_injector_library(
    provider_id: &str,
    provider_name: &str,
    library: ExternalAuthLibrary,
) -> Result<PathBuf> {
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let asset_name = sqlx::query_scalar::<_, String>(
        "SELECT asset_name FROM external_auth_libraries WHERE provider_id = ?",
    )
    .bind(provider_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(asset_name) = asset_name else {
        return Err(missing_external_auth_library(provider_name));
    };

    if validate_library_asset_name(&asset_name).is_err() {
        return Err(missing_external_auth_library(provider_name));
    }
    let path = authlib_injector_path(&libraries_dir, library, &asset_name);
    if !path.is_file() {
        return Err(missing_external_auth_library(provider_name));
    }

    tracing::debug!(
        provider = provider_id,
        asset = %asset_name,
        "[AR] Auth library selected"
    );
    Ok(path)
}

/// Lists valid library files installed for a provider.
pub async fn local_authlib_injector_libraries(
    library: ExternalAuthLibrary,
) -> Result<Vec<String>> {
    let state = State::get().await?;
    let mut entries = match fs::read_dir(authlib_injector_dir(
        &state.directories.libraries_dir(),
        library,
    ))
    .await
    {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let mut asset_names = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        let Some(asset_name) = file_name.to_str() else {
            continue;
        };
        if entry.file_type().await?.is_file()
            && validate_library_asset_name(asset_name).is_ok()
        {
            asset_names.push(asset_name.to_string());
        }
    }

    Ok(asset_names)
}

/// Downloads an exact remote asset and stores it as the provider selection.
pub async fn install_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> Result<()> {
    validate_library_asset_name(asset_name)?;
    let asset = fetch_external_auth_library_release(library)
        .await?
		.assets
		.authlib_injector
		.into_iter()
		.find(|asset| {
					asset.name == asset_name && is_authlib_injector_asset_name(&asset.name)
				})
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: format!("Library asset not found: {asset_name}"),
            }
            .as_error()
        })?;
    install_authlib_injector_asset(provider_id, library, asset).await?;

    Ok(())
}

/// Downloads and selects the newest provider library available remotely.
pub async fn install_latest_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
) -> Result<PathBuf> {
	let release = fetch_external_auth_library_release(library).await?;
	let asset = latest_authlib_injector_asset(release)?;

	install_authlib_injector_asset(provider_id, library, asset).await
}

async fn install_authlib_injector_asset(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset: ExternalAuthLibraryAsset,
) -> Result<PathBuf> {
    validate_library_asset_name(&asset.name)?;
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let directory = authlib_injector_dir(&libraries_dir, library);
    fs::create_dir_all(&directory).await?;
    let path = directory.join(&asset.name);

    tracing::debug!(
        provider = provider_id,
        asset = %asset.name,
        "[AR] Auth library download started"
    );
    let _ = emit_info("[AR] Installing auth library...").await;
    let bytes = fetch_bytes_from_url(asset.browser_download_url()?).await?;
    let relative_path = path
        .strip_prefix(&libraries_dir)?
        .to_string_lossy()
        .into_owned();
    write_file_to_libraries(&relative_path, &bytes).await?;
    save_authlib_injector_library_selection(provider_id, &asset.name).await?;

    Ok(path)
}

/// Stores a selection only when the exact local file exists.
pub async fn select_local_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> Result<bool> {
    validate_library_asset_name(asset_name)?;
    let state = State::get().await?;
    let path = authlib_injector_path(
        &state.directories.libraries_dir(),
        library,
        asset_name,
    );
    if !path.is_file() {
        return Ok(false);
    }

    save_authlib_injector_library_selection(provider_id, asset_name).await?;

    Ok(true)
}

async fn save_authlib_injector_library_selection(
    provider_id: &str,
    asset_name: &str,
) -> Result<()> {
    let state = State::get().await?;
    sqlx::query(
        r#"
		INSERT INTO external_auth_libraries (provider_id, asset_name)
		VALUES (?, ?)
		ON CONFLICT(provider_id) DO UPDATE SET asset_name = excluded.asset_name
		"#,
    )
    .bind(provider_id)
    .bind(asset_name)
    .execute(&state.pool)
    .await?;

    Ok(())
}

fn missing_external_auth_library(provider_name: &str) -> crate::Error {
    crate::ErrorKind::ExternalAuthLibraryNotInstalled {
        provider_name: provider_name.to_string(),
    }
    .as_error()
}

fn validate_library_asset_name(asset_name: &str) -> Result<()> {
    let path = Path::new(asset_name);
    let is_file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == asset_name);
	if !is_file_name
		|| asset_name.contains('\\')
		|| asset_name.chars().any(|character| character.is_ascii_control())
		|| !asset_name.to_ascii_lowercase().ends_with(".jar")
	{
        return Err(crate::ErrorKind::InputError(format!(
            "Invalid external authentication library asset: {asset_name}",
        ))
        .as_error());
    }

    Ok(())
}

fn authlib_injector_dir(
    libraries_dir: &Path,
    library: ExternalAuthLibrary,
) -> PathBuf {
    libraries_dir
        .join("astralrinth")
        .join(library.cache_directory)
}

fn authlib_injector_path(
    libraries_dir: &Path,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> PathBuf {
    authlib_injector_dir(libraries_dir, library).join(asset_name)
}

async fn fetch_external_auth_library_release(
    library: ExternalAuthLibrary,
) -> Result<ExternalAuthLibraryRelease> {
    Ok(reqwest::Client::new()
        .get(library.release_url)
		.header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?
        .json::<ExternalAuthLibraryRelease>()
        .await?)
}

fn latest_authlib_injector_asset(
	release: ExternalAuthLibraryRelease,
) -> Result<ExternalAuthLibraryAsset> {
	release
		.assets
		.authlib_injector
		.into_iter()
		.filter(|asset| asset.browser_download_url().is_ok())
		.filter_map(|asset| {
			library_version(&asset.name).map(|version| (asset, version))
		})
		.max_by(|(left_asset, left_version), (right_asset, right_version)| {
			left_version
				.cmp(right_version)
				.then_with(|| left_asset.name.cmp(&right_asset.name))
		})
		.map(|(asset, _)| asset)
		.ok_or_else(|| {
			crate::ErrorKind::ParseError {
				reason: "No downloadable compatible external authentication library was found"
					.to_string(),
			}
			.as_error()
		})
}

fn is_authlib_injector_asset_name(asset_name: &str) -> bool {
	if validate_library_asset_name(asset_name).is_err() {
		return false;
	}
	let name = asset_name.to_ascii_lowercase();
	name.strip_prefix("old_")
		.unwrap_or(&name)
		.starts_with("authlib-injector-")
}

fn library_version(asset_name: &str) -> Option<Vec<u64>> {
	if !is_authlib_injector_asset_name(asset_name) {
        return None;
    }

    asset_name
        .split(|character: char| {
            !character.is_ascii_digit() && character != '.'
        })
        .find_map(|candidate| {
            let version = candidate.trim_matches('.');
            if !version.contains('.') {
                return None;
            }

            version
                .split('.')
                .map(str::parse)
                .collect::<std::result::Result<Vec<u64>, _>>()
                .ok()
        })
}

/// Initialize the update launcher.
pub async fn init_update_launcher(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
) -> Result<()> {
    tracing::info!(
        file = local_filename,
        os = os_type,
        "[AR] Downloading launcher update"
    );

    if let Err(error) =
        update::get_resource(download_url, local_filename, os_type).await
    {
        tracing::error!(error = %error, "[AR] Launcher update failed");
    } else {
        tracing::info!("[AR] Launcher update ready");
        process::exit(0)
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

    fs::write(&output_path, bytes).await.map_err(|error| {
        tracing::error!(error = %error, "[AR] Library save failed");
        crate::ErrorKind::IOErrorOccurred {
            error: format!("Failed to save file: {error}"),
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
            timeout_seconds = TIMEOUT_SECONDS,
            "[AR] Download timed out"
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Download timed out after {TIMEOUT_SECONDS} seconds"
            )
            .to_string(),
        }
        .as_error()
    })?
    .map_err(|error| {
        tracing::error!(error = %error, "[AR] Download request failed");
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Request error: {error}"),
        }
        .as_error()
    })?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        tracing::error!(%status, "[AR] Download failed");
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to download file: HTTP {status}"),
        }
        .as_error());
    }

    response.bytes().await.map_err(|error| {
        tracing::error!(error = %error, "[AR] Download read failed");
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to read response bytes: {error}"),
        }
        .as_error()
    })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn grouped_library_release_accepts_optional_upstream_metadata() {
		let release: ExternalAuthLibraryRelease = serde_json::from_value(
			serde_json::json!({
				"total_downloads": 42,
				"assets": {
					"authlib": [{ "name": "authlib-9.0.jar" }],
					"authlib-injector": [{
						"name": "authlib-injector-1.2.jar",
						"arbitrary_upstream_field": true,
					}],
				},
			}),
		)
		.unwrap();
		assert_eq!(release.assets.authlib_injector.len(), 1);
		let asset = &release.assets.authlib_injector[0];
		assert_eq!(asset.name, "authlib-injector-1.2.jar");
		assert!(asset.browser_download_url.is_none());
		assert!(asset.browser_download_url().is_err());
	}

	#[test]
	fn latest_library_uses_only_downloadable_injector_assets() {
		let release: ExternalAuthLibraryRelease = serde_json::from_value(
			serde_json::json!({
				"tag_name": "release",
				"name": "Libraries",
				"total_downloads": 0,
				"assets": {
					"authlib": [{
						"name": "authlib-injector-999.0.jar",
						"browser_download_url": "https://example.com/wrong-group.jar",
					}],
					"authlib-injector": [
						{ "name": "authlib-injector-99.0.jar" },
						{ "name": "authlib-injector-98.0.jar", "browser_download_url": " " },
						{
							"name": "authlib-100.0.jar",
							"browser_download_url": "https://example.com/wrong-library.jar",
						},
						{
							"name": "authlib-injector-1.9.jar",
							"browser_download_url": "https://example.com/1.9.jar",
						},
						{
							"name": "OLD_AUTHLIB-INJECTOR-1.10.JAR",
							"browser_download_url": "https://example.com/1.10.jar",
						},
					],
				},
			}),
		)
		.unwrap();
		let asset = latest_authlib_injector_asset(release).unwrap();
		assert_eq!(asset.name, "OLD_AUTHLIB-INJECTOR-1.10.JAR");
		assert_eq!(asset.browser_download_url().unwrap(), "https://example.com/1.10.jar");
	}

	#[test]
	fn missing_null_or_blank_download_urls_cannot_be_installed() {
		for metadata in [
			serde_json::json!({ "name": "authlib-injector-1.2.jar" }),
			serde_json::json!({
				"name": "authlib-injector-1.2.jar",
				"browser_download_url": null,
			}),
			serde_json::json!({
				"name": "authlib-injector-1.2.jar",
				"browser_download_url": " ",
			}),
		] {
			let asset: ExternalAuthLibraryAsset =
				serde_json::from_value(metadata).unwrap();
			assert!(asset.browser_download_url().is_err());
		}
	}

	#[test]
	fn empty_injector_group_does_not_fall_back_to_authlib() {
		let release: ExternalAuthLibraryRelease = serde_json::from_value(
			serde_json::json!({
				"assets": {
					"authlib": [{
						"name": "authlib-injector-1.2.jar",
						"browser_download_url": "https://example.com/wrong-group.jar",
					}],
					"authlib-injector": [],
				},
			}),
		)
		.unwrap();
		assert!(latest_authlib_injector_asset(release).is_err());
	}

	#[test]
	fn library_release_rejects_flat_or_missing_injector_assets() {
		for value in [
			serde_json::json!({ "assets": [] }),
			serde_json::json!({ "assets": { "authlib": [] } }),
			serde_json::json!({ "assets": { "authlib-injector": {} } }),
		] {
			assert!(
							serde_json::from_value::<ExternalAuthLibraryRelease>(value).is_err()
						);
		}
	}

	#[test]
	fn injector_asset_names_match_the_public_api_safely() {
		for name in [
					"authlib-injector-1.2.jar",
					"OLD_AUTHLIB-INJECTOR-1.10.JAR",
				] {
			assert!(is_authlib_injector_asset_name(name));
			assert!(library_version(name).is_some());
		}
		for name in [
			"authlib-1.2.jar",
			"prefix-authlib-injector-1.2.jar",
			"authlib-injector-1.2.zip",
			"authlib-injector-../1.2.jar",
			"old_authlib-injector-..\\1.2.jar",
			"authlib-injector-1.2\0.jar",
		] {
			assert!(!is_authlib_injector_asset_name(name), "{name:?}");
			assert!(library_version(name).is_none(), "{name:?}");
		}
	}
}
