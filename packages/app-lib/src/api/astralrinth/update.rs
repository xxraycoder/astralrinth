use crate::util::fetch::NO_TIMEOUT_REQWEST_CLIENT;
use std::path::{Component, Path, PathBuf};
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const EMPTY_OPEN_ARGS: &[&str] = &[];

type UpdateError = Box<dyn std::error::Error + Send + Sync>;

pub(crate) async fn get_resource(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
) -> Result<(), UpdateError> {
    let url = validated_https_url(download_url)?;
    let file_name = validated_file_name(local_filename)?;
    let download_dir = dirs::download_dir()
        .ok_or("[AR] • Failed to determine download directory")?;
    let full_path = download_dir.join(file_name);

    download_file(&url, &full_path).await?;
    tracing::info!("[AR] • File downloaded to: {:?}", full_path);

    let result = match os_type.to_lowercase().as_str() {
        "windows" => handle_windows_file(&full_path).await,
        "macos" => open_macos_file(&full_path).await,
        _ => open_default(&full_path).await,
    };

    match &result {
        Ok(_) => tracing::info!("[AR] • File opened successfully!"),
        Err(e) => tracing::error!("[AR] • Failed to open file: {e}"),
    }

    result
}

fn validated_https_url(
    download_url: &str,
) -> Result<reqwest::Url, UpdateError> {
    let url = reqwest::Url::parse(download_url)?;
    if url.scheme() != "https" {
        return Err("[AR] • Update download URL must use HTTPS".into());
    }
    Ok(url)
}

fn validated_file_name(local_filename: &str) -> Result<&str, UpdateError> {
    let mut components = Path::new(local_filename).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(local_filename),
        _ => {
            Err("[AR] • Update file name must not contain path components"
                .into())
        }
    }
}

/// Streams the response into a `.part` file and renames it only after the whole body was written,
/// so an interrupted download never leaves a truncated installer under the final name.
async fn download_file(
    url: &reqwest::Url,
    destination: &Path,
) -> Result<(), UpdateError> {
    let mut partial_name = destination.as_os_str().to_owned();
    partial_name.push(".part");
    let partial_path = PathBuf::from(partial_name);

    match stream_to_file(url, &partial_path).await {
        Ok(()) => {
            tokio::fs::rename(&partial_path, destination).await?;
            Ok(())
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&partial_path).await;
            Err(error)
        }
    }
}

async fn stream_to_file(
    url: &reqwest::Url,
    path: &Path,
) -> Result<(), UpdateError> {
    let mut response = NO_TIMEOUT_REQWEST_CLIENT
        .get(url.clone())
        .send()
        .await?
        .error_for_status()?;
    let mut file = AsyncFile::create(path).await?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(())
}

async fn handle_windows_file(path: &Path) -> Result<(), UpdateError> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if filename.ends_with(".exe") || filename.ends_with(".msi") {
        tracing::info!("[AR] • Detected installer: {}", filename);
        run_windows_installer(path).await
    } else {
        open_windows_folder(path).await
    }
}

async fn run_windows_installer(path: &Path) -> Result<(), UpdateError> {
    let is_msi = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("msi"));

    let status = if is_msi {
        Command::new("msiexec")
            .arg("/i")
            .arg(path)
            .arg("/quiet")
            .status()
            .await?
    } else {
        Command::new(path).status().await?
    };

    if status.success() {
        tracing::info!("[AR] • Installer started successfully.");
        return Ok(());
    }

    tracing::error!("[AR] • Installer failed. Exit code: {:?}", status.code());
    if let Err(error) = open_windows_folder(path).await {
        tracing::error!("[AR] • Failed to open folder: {error}");
    }
    Err(format!("Installer exited with code {:?}", status.code()).into())
}

async fn open_windows_folder(path: &Path) -> Result<(), UpdateError> {
    open_in_file_manager(path, &[("explorer", EMPTY_OPEN_ARGS)]).await
}

async fn open_macos_file(path: &Path) -> Result<(), UpdateError> {
    let status = Command::new("open").arg(path).status().await?;

    if !status.success() {
        Err(format!("Exit code: {:?}", status.code()).into())
    } else {
        Ok(())
    }
}

async fn open_default(path: &Path) -> Result<(), UpdateError> {
    open_in_file_manager(
        path,
        &[
            ("xdg-open", EMPTY_OPEN_ARGS),
            ("gio", &["open"]),
            ("kioclient5", &["exec"]),
            ("kioclient", &["exec"]),
            ("kde-open", EMPTY_OPEN_ARGS),
        ],
    )
    .await
}

async fn open_in_file_manager(
    path: &Path,
    commands: &[(&str, &[&str])],
) -> Result<(), UpdateError> {
    let folder = path.parent().unwrap_or(path);
    let mut last_error = None;

    for (command, args) in commands {
        let mut process = Command::new(command);
        process.args(*args).arg(folder);

        match process.status().await {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => {
                last_error = Some(format!(
                    "{command} exited with code {:?}",
                    status.code()
                ));
            }
            Err(error) => {
                last_error = Some(format!("{command} failed: {error}"));
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| "No file manager command succeeded".to_string())
        .into())
}
