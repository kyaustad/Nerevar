use crate::types::{Mode, NerevarConfig, OpenMWConfig, UpdateCheckResponse};
use crate::utils::{find_tes3mp_folder, get_appdata_dir};
use open;
use std::fs::{self, File};
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use tauri::Emitter;
use zip::ZipArchive;

#[tauri::command]
pub async fn download_latest_windows_release() -> Result<String, String> {
    // Use temp directory to avoid Tauri rebuilds
    let temp_dir = std::env::temp_dir();

    let zip_filename = "tes3mp_latest.zip";
    let extract_dir = "tes3mp_extracted";

    let zip_path = temp_dir.join(zip_filename);
    let extract_path = temp_dir.join(extract_dir);

    log::info!("Using temp directory: {}", temp_dir.display());
    log::info!("Zip will be saved to: {}", zip_path.display());
    log::info!("Will extract to: {}", extract_path.display());

    //Step 0: get latest release url from nerevar-api which returns a body json { url: string, version: string }
    let api_url =
        std::env::var("NEREVAR_API_URL").expect("NEREVAR_API_URL environment variable must be set");
    let version = match reqwest::get(format!("{}/releases/tes3mp", api_url)).await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(url_response) => match url_response["version"].as_str() {
                Some(version) => {
                    log::info!("Successfully retrieved version from API: {}", version);
                    version.to_string()
                }
                None => {
                    log::warn!("API returned invalid response format, using fallback version");
                    "".to_string()
                }
            },
            Err(e) => {
                log::warn!(
                    "Failed to parse API response: {}, using fallback version",
                    e
                );
                "".to_string()
            }
        },
        Err(e) => {
            log::warn!(
                "Failed to connect to API ({}): {}, using fallback version",
                api_url,
                e
            );
            "".to_string()
        }
    };
    let url = match reqwest::get(format!("{}/releases/tes3mp", api_url)).await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(url_response) => match url_response["url"].as_str() {
                Some(url) => {
                    log::info!("Successfully retrieved URL from API: {}", url);
                    url.to_string()
                }
                None => {
                    log::warn!("API returned invalid response format, using fallback URL");
                    "".to_string()
                }
            },
            Err(e) => {
                log::warn!("Failed to parse API response: {}, using fallback URL", e);
                "".to_string()
            }
        },
        Err(e) => {
            log::warn!(
                "Failed to connect to API ({}): {}, using fallback URL",
                api_url,
                e
            );
            "".to_string()
        }
    };
    log::info!("Latest release url: {}", url);

    // Step 1: Download the zip file
    log::info!("Downloading TES3MP from: {}", url);
    let response = reqwest::get(url.as_str())
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    // Step 2: Save the zip file
    fs::write(&zip_path, bytes).map_err(|e| format!("Failed to save zip file: {}", e))?;

    log::info!("Zip file saved as: {}", zip_path.display());

    // Step 3: Extract the zip file
    let zip_file = File::open(&zip_path).map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive = ZipArchive::new(BufReader::new(zip_file))
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    // Create extraction directory
    if extract_path.exists() {
        log::info!(
            "Removing existing extraction directory: {}",
            extract_path.display()
        );
        fs::remove_dir_all(&extract_path)
            .map_err(|e| format!("Failed to remove existing extraction directory: {}", e))?;
    }

    log::info!("Extracting zip file to: {}", extract_path.display());
    archive
        .extract(&extract_path)
        .map_err(|e| format!("Failed to extract zip file: {}", e))?;

    log::info!("Zip file extracted to: {}", extract_path.display());

    // Step 4: Delete the zip file
    fs::remove_file(&zip_path).map_err(|e| format!("Failed to delete zip file: {}", e))?;

    log::info!("Zip file deleted: {}", zip_path.display());

    // Step 5: Check if tes3mp.exe is directly in the extracted folder
    let exe_path = extract_path.join("tes3mp.exe");

    log::info!(
        "Checking for tes3mp.exe directly in: {}",
        extract_path.display()
    );

    let (tes3mp_folder, final_exe_path) = if exe_path.exists() {
        log::info!(
            "Found tes3mp.exe directly in extraction folder: {}",
            exe_path.display()
        );
        log::info!(
            "Using extraction folder as TES3MP folder: {}",
            extract_path.display()
        );
        (extract_path.clone(), exe_path)
    } else {
        // Step 6: Look for TES3MP folder (should be named something like tes3mp.Win64.release.0.8.1)
        log::info!(
            "tes3mp.exe not found directly, looking for TES3MP folder in: {}",
            extract_path.display()
        );
        let tes3mp_folder = find_tes3mp_folder(&extract_path)?;
        log::info!("Found TES3MP folder: {}", tes3mp_folder.display());
        let exe_path = tes3mp_folder.join("tes3mp.exe");
        (tes3mp_folder, exe_path)
    };

    log::info!("Checking for tes3mp.exe at: {}", final_exe_path.display());
    if !final_exe_path.exists() {
        // List all files in the folder for debugging
        log::error!("tes3mp.exe not found in: {}", tes3mp_folder.display());
        if let Ok(entries) = fs::read_dir(&tes3mp_folder) {
            log::info!("Files in TES3MP folder:");
            for entry in entries {
                if let Ok(entry) = entry {
                    log::info!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
        return Err(format!(
            "tes3mp.exe not found in: {}",
            tes3mp_folder.display()
        ));
    }

    log::info!("Found tes3mp.exe at: {}", final_exe_path.display());
    log::info!(
        "TES3MP installation verified in: {}",
        tes3mp_folder.display()
    );

    // Step 7: Move TES3MP to permanent AppData location
    let appdata_dir = get_appdata_dir()?;
    let permanent_tes3mp_path = appdata_dir.join("TES3MP");

    log::info!(
        "Moving TES3MP to permanent location: {}",
        permanent_tes3mp_path.display()
    );

    // Remove existing TES3MP folder if it exists
    if permanent_tes3mp_path.exists() {
        log::info!(
            "Removing existing TES3MP installation: {}",
            permanent_tes3mp_path.display()
        );
        fs::remove_dir_all(&permanent_tes3mp_path)
            .map_err(|e| format!("Failed to remove existing TES3MP installation: {}", e))?;
    }

    // Move the extracted folder to AppData
    fs::rename(&tes3mp_folder, &permanent_tes3mp_path)
        .map_err(|e| format!("Failed to move TES3MP to AppData: {}", e))?;

    log::info!(
        "TES3MP successfully moved to: {}",
        permanent_tes3mp_path.display()
    );

    // Step 8: Clean up temp extraction directory
    if extract_path.exists() {
        log::info!(
            "Cleaning up temp extraction directory: {}",
            extract_path.display()
        );
        fs::remove_dir_all(&extract_path)
            .map_err(|e| format!("Failed to clean up temp extraction directory: {}", e))?;
    }

    // Step 9: Create/update Nerevar config
    let config_path = appdata_dir.join("config.json");
    let config = NerevarConfig {
        tes3mp_path: permanent_tes3mp_path.to_string_lossy().to_string(),
        version: version,
        last_updated: chrono::Utc::now().to_rfc3339(),
        mode: Some(Mode::Player),
    };

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Nerevar config updated: {}", config_path.display());

    Ok(format!(
        "TES3MP successfully installed to: {}",
        permanent_tes3mp_path.display()
    ))
}

#[tauri::command]
pub fn get_nerevar_config() -> Result<Option<NerevarConfig>, String> {
    crate::config::get_nerevar_config()
}

#[tauri::command]
pub fn get_openmw_config() -> Result<Option<OpenMWConfig>, String> {
    crate::config::get_openmw_config()
}

#[tauri::command]
pub fn set_mode(mode: Mode) -> Result<String, String> {
    crate::config::set_mode(mode)
}

#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    // Get the app version from the Tauri context
    let version = env!("CARGO_PKG_VERSION");
    Ok(version.to_string())
}

#[tauri::command]
pub async fn ping_server_tcp(ip: String, port: u16) -> Option<u128> {
    let addr = format!("{}:{}", ip, port);
    let start = Instant::now();
    match TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_secs(2)) {
        Ok(_) => Some(start.elapsed().as_millis()),
        Err(_) => None,
    }
}

// Placeholder for other commands - will be added in next steps
#[tauri::command]
pub async fn run_openmw_wizard(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running OpenMW wizard");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the OpenMW wizard executable
    let openmw_wizard_path = Path::new(&tes3mp_path).join("openmw-wizard.exe");

    // Check if the wizard executable exists
    if !openmw_wizard_path.exists() {
        return Err(format!(
            "OpenMW wizard not found at: {}",
            openmw_wizard_path.display()
        ));
    }

    log::info!("Running OpenMW wizard at: {}", openmw_wizard_path.display());

    // Spawn the OpenMW wizard process
    let mut child = std::process::Command::new(&openmw_wizard_path)
        .spawn()
        .map_err(|e| format!("Failed to run OpenMW wizard: {}", e))?;

    let pid = child.id();
    log::info!("OpenMW wizard started successfully (PID: {})", pid);

    // Send initial event that wizard started
    app_handle
        .emit("openmw-wizard-started", &pid)
        .map_err(|e| format!("Failed to emit wizard started event: {}", e))?;

    // Spawn a task to monitor the process
    tokio::spawn(async move {
        // Wait for the process to complete
        match child.wait() {
            Ok(status) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": status.success(),
                    "exit_code": status.code(),
                    "message": if status.success() {
                        "OpenMW wizard completed successfully"
                    } else {
                        "OpenMW wizard exited with an error"
                    }
                });

                log::info!(
                    "OpenMW wizard (PID: {}) exited with status: {:?}",
                    pid,
                    status.code()
                );

                // Emit the completion event
                if let Err(e) = app_handle.emit("openmw-wizard-exited", &event_data) {
                    log::error!("Failed to emit wizard exited event: {}", e);
                }
            }
            Err(e) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": false,
                    "exit_code": None::<i32>,
                    "message": format!("Failed to wait for OpenMW wizard: {}", e)
                });

                log::error!("Failed to wait for OpenMW wizard (PID: {}): {}", pid, e);

                // Emit the error event
                if let Err(emit_err) = app_handle.emit("openmw-wizard-exited", &event_data) {
                    log::error!("Failed to emit wizard error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!("OpenMW wizard started successfully (PID: {})", pid))
}

#[tauri::command]
pub async fn run_openmw_launcher(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running OpenMW launcher");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the OpenMW launcher executable
    let openmw_launcher_path = Path::new(&tes3mp_path).join("openmw-launcher.exe");

    // Check if the launcher executable exists
    if !openmw_launcher_path.exists() {
        return Err(format!(
            "OpenMW launcher not found at: {}",
            openmw_launcher_path.display()
        ));
    }

    log::info!(
        "Running OpenMW launcher at: {}",
        openmw_launcher_path.display()
    );

    // Spawn the OpenMW launcher process
    let mut child = std::process::Command::new(&openmw_launcher_path)
        .spawn()
        .map_err(|e| format!("Failed to run OpenMW launcher: {}", e))?;

    let pid = child.id();
    log::info!("OpenMW launcher started successfully (PID: {})", pid);

    // Send initial event that launcher started
    app_handle
        .emit("openmw-launcher-started", &pid)
        .map_err(|e| format!("Failed to emit launcher started event: {}", e))?;

    // Spawn a task to monitor the process
    tokio::spawn(async move {
        // Wait for the process to complete
        match child.wait() {
            Ok(status) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": status.success(),
                    "exit_code": status.code(),
                    "message": if status.success() {
                        "OpenMW launcher completed successfully"
                    } else {
                        "OpenMW launcher exited with an error"
                    }
                });

                log::info!(
                    "OpenMW launcher (PID: {}) exited with status: {:?}",
                    pid,
                    status.code()
                );

                // Emit the completion event
                if let Err(e) = app_handle.emit("openmw-launcher-exited", &event_data) {
                    log::error!("Failed to emit launcher exited event: {}", e);
                }
            }
            Err(e) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": false,
                    "exit_code": None::<i32>,
                    "message": format!("Failed to wait for OpenMW launcher: {}", e)
                });

                log::error!("Failed to wait for OpenMW launcher (PID: {}): {}", pid, e);

                // Emit the error event
                if let Err(emit_err) = app_handle.emit("openmw-launcher-exited", &event_data) {
                    log::error!("Failed to emit launcher error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!(
        "OpenMW launcher started successfully (PID: {})",
        pid
    ))
}

#[tauri::command]
pub async fn check_for_tes3mp_update() -> Result<UpdateCheckResponse, String> {
    // Get config and check version, then hit the api and check the returned version, if the returned version is greater than the current version, return true, otherwise return false
    let config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;
    let config = config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let current_version = config.version;
    let api_url =
        std::env::var("NEREVAR_API_URL").expect("NEREVAR_API_URL environment variable must be set");
    let response = reqwest::get(format!("{}/releases/tes3mp", api_url))
        .await
        .map_err(|e| format!("Failed to check for TES3MP update: {}", e))?;
    let response = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse TES3MP update response: {}", e))?;
    let latest_version = response["version"].as_str().unwrap_or("0.0.0");
    let download_url = response["url"].as_str().map(|s| s.to_string());
    log::info!("Latest version: {}", latest_version);
    log::info!("Current version: {}", current_version);
    Ok(UpdateCheckResponse {
        update_available: current_version != latest_version,
        version: latest_version.to_string(),
        url: download_url,
    })
}

#[tauri::command]
pub async fn check_for_app_update() -> Result<UpdateCheckResponse, String> {
    // Get the current app version from the built-in version
    let current_version = env!("CARGO_PKG_VERSION");

    let api_url =
        std::env::var("NEREVAR_API_URL").expect("NEREVAR_API_URL environment variable must be set");
    let response = reqwest::get(format!("{}/releases/nerevar", api_url))
        .await
        .map_err(|e| format!("Failed to check for Nerevar update: {}", e))?;
    let response = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse Nerevar update response: {}", e))?;
    let latest_version = response["version"].as_str().unwrap_or("0.0.0");
    let download_url = response["url"].as_str().map(|s| s.to_string());
    log::info!("Latest version: {}", latest_version);
    log::info!("Current version: {}", current_version);
    Ok(UpdateCheckResponse {
        update_available: current_version != latest_version,
        version: latest_version.to_string(),
        url: download_url,
    })
}

#[tauri::command]
pub async fn download_app_update(download_url: String) -> Result<String, String> {
    log::info!("Starting download from: {}", download_url);

    // Create a temporary file for the download
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join("nerevar_update.exe");

    // Download the file
    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download content: {}", e))?;

    // Write directly to the temp file
    fs::write(&temp_file_path, &content)
        .map_err(|e| format!("Failed to write downloaded content: {}", e))?;

    let temp_path_str = temp_file_path.to_string_lossy().to_string();

    log::info!("Download completed to: {}", temp_path_str);
    Ok(temp_path_str)
}

#[tauri::command]
pub async fn apply_app_update(temp_file_path: String) -> Result<String, String> {
    log::info!("Applying update from: {}", temp_file_path);

    // Get the current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    let current_exe_str = current_exe.to_string_lossy().to_string();
    log::info!("Current executable: {}", current_exe_str);

    // Create a backup of the current executable
    let backup_path = format!("{}.backup", current_exe_str);
    fs::copy(&current_exe, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    log::info!("Created backup at: {}", backup_path);

    // Create a batch script to handle the replacement after exit
    let temp_dir = std::env::temp_dir();
    let batch_script_path = temp_dir.join("nerevar_update.bat");

    let batch_content = format!(
        r#"@echo off
timeout /t 2 /nobreak >nul
copy "{}" "{}"
if %errorlevel% equ 0 (
    del "{}"
    del "%~f0"
    start "" "{}"
) else (
    echo Update failed, restoring backup...
    copy "{}" "{}"
    del "{}"
    del "%~f0"
)
"#,
        temp_file_path,  // Source (new exe)
        current_exe_str, // Destination (current exe)
        temp_file_path,  // Delete temp file
        current_exe_str, // Start new exe
        backup_path,     // Restore from backup
        current_exe_str, // Restore to current exe
        backup_path      // Delete backup
    );

    fs::write(&batch_script_path, batch_content)
        .map_err(|e| format!("Failed to create batch script: {}", e))?;

    log::info!("Created batch script at: {}", batch_script_path.display());

    // Start the batch script
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", &batch_script_path.to_string_lossy()]);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // On Windows, we need to detach the process
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x00000008); // CREATE_NEW_PROCESS_GROUP
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to start update script: {}", e))?;

    log::info!("Update script started, exiting current process");

    // Exit the current process
    std::process::exit(0);
}

#[tauri::command]
pub async fn run_tes3mp_browser(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running TES3MP browser");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the TES3MP browser executable
    let server_browser_path = Path::new(&tes3mp_path).join("tes3mp-browser.exe");

    // Check if the TES3MP browser executable exists
    if !server_browser_path.exists() {
        return Err(format!(
            "TES3MP browser not found at: {}",
            server_browser_path.display()
        ));
    }

    log::info!(
        "Running TES3MP browser at: {}",
        server_browser_path.display()
    );

    // Spawn the TES3MP browser process
    let mut child = std::process::Command::new(&server_browser_path)
        .spawn()
        .map_err(|e| format!("Failed to run TES3MP browser: {}", e))?;

    let pid = child.id();
    log::info!("TES3MP browser started successfully (PID: {})", pid);

    // Send initial event that TES3MP browser started
    app_handle
        .emit("tes3mp-browser-started", &pid)
        .map_err(|e| format!("Failed to emit TES3MP browser started event: {}", e))?;

    // Spawn a task to monitor the process
    tokio::spawn(async move {
        // Wait for the process to complete
        match child.wait() {
            Ok(status) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": status.success(),
                    "exit_code": status.code(),
                    "message": if status.success() {
                        "TES3MP browser completed successfully"
                    } else {
                        "TES3MP browser exited with an error"
                    }
                });

                log::info!(
                    "TES3MP browser (PID: {}) exited with status: {:?}",
                    pid,
                    status.code()
                );

                // Emit the completion event
                if let Err(e) = app_handle.emit("tes3mp-browser-exited", &event_data) {
                    log::error!("Failed to emit TES3MP browser exited event: {}", e);
                }
            }
            Err(e) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": false,
                    "exit_code": None::<i32>,
                    "message": format!("Failed to wait for TES3MP browser: {}", e)
                });

                log::error!("Failed to wait for TES3MP browser (PID: {}): {}", pid, e);

                // Emit the error event
                if let Err(emit_err) = app_handle.emit("tes3mp-browser-exited", &event_data) {
                    log::error!("Failed to emit TES3MP browser error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!(
        "TES3MP browser started successfully (PID: {})",
        pid
    ))
}

#[tauri::command]
pub async fn run_tes3mp(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running TES3MP");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the TES3MP executable
    let tes3mp_path = Path::new(&tes3mp_path).join("tes3mp.exe");

    // Check if the TES3MP executable exists
    if !tes3mp_path.exists() {
        return Err(format!("TES3MP not found at: {}", tes3mp_path.display()));
    }

    log::info!("Running TES3MP at: {}", tes3mp_path.display());

    // Spawn the TES3MP process
    let mut child = std::process::Command::new(&tes3mp_path)
        .spawn()
        .map_err(|e| format!("Failed to run TES3MP: {}", e))?;

    let pid = child.id();
    log::info!("TES3MP started successfully (PID: {})", pid);

    // Send initial event that TES3MP started
    app_handle
        .emit("tes3mp-started", &pid)
        .map_err(|e| format!("Failed to emit TES3MP started event: {}", e))?;

    // Spawn a task to monitor the process
    tokio::spawn(async move {
        // Wait for the process to complete
        match child.wait() {
            Ok(status) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": status.success(),
                    "exit_code": status.code(),
                    "message": if status.success() {
                        "TES3MP completed successfully"
                    } else {
                        "TES3MP exited with an error"
                    }
                });

                log::info!(
                    "TES3MP (PID: {}) exited with status: {:?}",
                    pid,
                    status.code()
                );

                // Emit the completion event
                if let Err(e) = app_handle.emit("tes3mp-exited", &event_data) {
                    log::error!("Failed to emit TES3MP exited event: {}", e);
                }
            }
            Err(e) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": false,
                    "exit_code": None::<i32>,
                    "message": format!("Failed to wait for TES3MP: {}", e)
                });

                log::error!("Failed to wait for TES3MP (PID: {}): {}", pid, e);

                // Emit the error event
                if let Err(emit_err) = app_handle.emit("tes3mp-exited", &event_data) {
                    log::error!("Failed to emit TES3MP error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!("TES3MP started successfully (PID: {})", pid))
}

#[tauri::command]
pub async fn set_tes3mp_client_config(
    ip: String,
    port: u16,
    password: String,
) -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP client config file
    let config_path = appdata_dir.join("TES3MP").join("tes3mp-client-default.cfg");

    // Check if the config file exists
    if !config_path.exists() {
        return Err(format!(
            "TES3MP client config file not found at: {}",
            config_path.display()
        ));
    }

    // Read the existing config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read TES3MP client config: {}", e))?;

    // Parse and update the config
    let updated_content =
        crate::config::update_config_values(&config_content, &ip, port, &password)?;

    // Write the updated config back to the file
    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write updated TES3MP client config: {}", e))?;

    log::info!(
        "Successfully updated TES3MP client config with IP: {}, Port: {}, Password: {}",
        ip,
        port,
        if password.is_empty() { "empty" } else { "set" }
    );

    Ok(true)
}

#[tauri::command]
pub async fn get_tes3mp_server_config() -> Result<serde_json::Value, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir.join("TES3MP").join("tes3mp-server-default.cfg");

    // Check if the config file exists
    if !config_path.exists() {
        return Err(format!(
            "TES3MP server config file not found at: {}",
            config_path.display()
        ));
    }

    // Read the config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read TES3MP server config: {}", e))?;

    // Parse the config content
    let parsed_config = crate::parsers::parse_server_config(&config_content)?;

    // Convert to JSON
    let json_config = serde_json::to_value(parsed_config)
        .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

    Ok(json_config)
}

#[tauri::command]
pub async fn set_tes3mp_server_config(config: serde_json::Value) -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir.join("TES3MP").join("tes3mp-server-default.cfg");

    // Check if the config file exists
    if !config_path.exists() {
        return Err(format!(
            "TES3MP server config file not found at: {}",
            config_path.display()
        ));
    }

    // Read the existing config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read TES3MP server config: {}", e))?;

    // Parse and update the config
    let updated_content = crate::config::update_server_config_values(&config_content, &config)?;

    // Write the updated config back to the file
    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write updated TES3MP server config: {}", e))?;

    log::info!("Successfully updated TES3MP server config");
    Ok(true)
}

#[tauri::command]
pub async fn get_tes3mp_server_settings() -> Result<serde_json::Value, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir
        .join("TES3MP")
        .join("server")
        .join("scripts")
        .join("config.lua");

    // Check if the config file exists
    if !config_path.exists() {
        return Err(format!(
            "TES3MP server settings file not found at: {}",
            config_path.display()
        ));
    }

    // Read the config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read TES3MP server settings: {}", e))?;

    // Parse the config content
    let parsed_config = crate::parsers::parse_server_settings(&config_content)?;

    // Convert to JSON
    let json_config = serde_json::to_value(parsed_config)
        .map_err(|e| format!("Failed to serialize settings to JSON: {}", e))?;

    Ok(json_config)
}

#[tauri::command]
pub async fn set_tes3mp_server_settings(settings: serde_json::Value) -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir
        .join("TES3MP")
        .join("server")
        .join("scripts")
        .join("config.lua");

    // Parse the JSON settings into our struct
    let server_settings: crate::types::ServerSettings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

    // Read the existing config file content
    let existing_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read existing TES3MP server settings: {}", e))?;

    // Update the config content with new settings
    let updated_content =
        crate::parsers::update_server_settings(&existing_content, &server_settings)?;

    // Write the updated config file
    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write TES3MP server settings: {}", e))?;

    log::info!("TES3MP server settings saved successfully");
    Ok(true)
}

#[tauri::command]
pub async fn run_tes3mp_server(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running TES3MP server");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the TES3MP server executable
    let server_path = Path::new(&tes3mp_path).join("tes3mp-server.exe");

    // Check if the TES3MP server executable exists
    if !server_path.exists() {
        return Err(format!(
            "TES3MP server not found at: {}",
            server_path.display()
        ));
    }

    log::info!("Running TES3MP server at: {}", server_path.display());

    // Spawn the TES3MP server process in its own terminal window
    // Use PowerShell to properly handle the start command with quotes
    let mut child = std::process::Command::new("powershell.exe")
        .args([
            "-Command",
            &format!(
                "Start-Process -FilePath '{}' -WindowStyle Normal -PassThru | Wait-Process",
                server_path.to_string_lossy()
            ),
        ])
        .spawn()
        .map_err(|e| format!("Failed to run TES3MP server in terminal: {}", e))?;

    let pid = child.id();
    log::info!(
        "TES3MP server started successfully in terminal (PID: {})",
        pid
    );

    // Send initial event that TES3MP server started
    app_handle
        .emit("tes3mp-server-started", &pid)
        .map_err(|e| format!("Failed to emit TES3MP server started event: {}", e))?;

    // Spawn a task to monitor the process
    tokio::spawn(async move {
        // Wait for the process to complete
        match child.wait() {
            Ok(status) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": status.success(),
                    "exit_code": status.code(),
                    "message": if status.success() {
                        "TES3MP server completed successfully"
                    } else {
                        "TES3MP server exited with an error"
                    }
                });

                log::info!(
                    "TES3MP server (PID: {}) exited with status: {:?}",
                    pid,
                    status.code()
                );

                // Emit the completion event
                if let Err(e) = app_handle.emit("tes3mp-server-exited", &event_data) {
                    log::error!("Failed to emit TES3MP server exited event: {}", e);
                }
            }
            Err(e) => {
                let event_data = serde_json::json!({
                    "pid": pid,
                    "success": false,
                    "exit_code": None::<i32>,
                    "message": format!("Failed to wait for TES3MP server: {}", e)
                });

                log::error!("Failed to wait for TES3MP server (PID: {}): {}", pid, e);

                // Emit the error event
                if let Err(emit_err) = app_handle.emit("tes3mp-server-exited", &event_data) {
                    log::error!("Failed to emit TES3MP server error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!("TES3MP server started successfully (PID: {})", pid))
}

#[tauri::command]
pub async fn open_config_lua_in_explorer() -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir
        .join("TES3MP")
        .join("server")
        .join("scripts")
        .join("config.lua");

    // Check if the config file exists
    if !config_path.exists() {
        return Err(format!(
            "TES3MP server settings file not found at: {}",
            config_path.display()
        ));
    }

    // Open the file in the default file explorer
    open::that(&config_path)
        .map_err(|e| format!("Failed to open config.lua in explorer: {}", e))?;

    log::info!("Opened config.lua in explorer: {}", config_path.display());
    Ok(true)
}

#[tauri::command]
pub async fn open_nerevar_appdata_dir_in_explorer() -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Open the directory in the default file explorer
    open::that(&appdata_dir).map_err(|e| {
        format!(
            "Failed to open Nerevar AppData directory in explorer: {}",
            e
        )
    })?;

    log::info!(
        "Opened Nerevar AppData directory in explorer: {}",
        appdata_dir.display()
    );
    Ok(true)
}
