use dirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use tauri::Emitter;
use zip::ZipArchive;

mod server_settings;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Player,
    Server,
}

#[derive(Serialize, Deserialize)]
struct UpdateCheckResponse {
    update_available: bool,
    version: String,
    url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct NerevarConfig {
    tes3mp_path: String,
    version: String,
    last_updated: String,
    mode: Option<Mode>,
}

// Use a flexible map for OpenMW config since it can contain any settings
type OpenMWConfig = std::collections::HashMap<String, serde_json::Value>;

fn get_documents_folder() -> Result<std::path::PathBuf, String> {
    if let Some(documents_dir) = dirs::document_dir() {
        log::info!("Documents directory: {}", documents_dir.display());
        return Ok(documents_dir);
    } else {
        Err(format!("Failed to get documents directory"))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::dotenv().ok();
    let api_url =
        std::env::var("NEREVAR_API_URL").expect("NEREVAR_API_URL environment variable must be set");
    log::info!("Nerevar API URL: {}", api_url);
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .invoke_handler(tauri::generate_handler![
            download_latest_windows_release,
            get_nerevar_config,
            get_openmw_config,
            run_openmw_wizard,
            run_openmw_launcher,
            check_for_tes3mp_update,
            set_mode,
            get_app_version,
            check_for_app_update,
            download_app_update,
            apply_app_update,
            run_tes3mp_browser,
            run_tes3mp,
            ping_server_tcp,
            set_tes3mp_client_config,
            get_tes3mp_server_config,
            set_tes3mp_server_config,
            get_tes3mp_server_settings,
            set_tes3mp_server_settings,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn download_latest_windows_release() -> Result<String, String> {
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
        (extract_path, exe_path)
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

    log::info!("TES3MP moved to: {}", permanent_tes3mp_path.display());

    // Step 8: Create or update config.json
    let config_path = appdata_dir.join("config.json");

    let config = if config_path.exists() {
        // Load existing config and update only necessary fields
        let existing_config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read existing config: {}", e))?;

        let mut existing_config: NerevarConfig = serde_json::from_str(&existing_config_content)
            .map_err(|e| format!("Failed to parse existing config: {}", e))?;

        // Update only the fields we need to
        existing_config.tes3mp_path = permanent_tes3mp_path.to_string_lossy().to_string();
        existing_config.version = version;
        existing_config.last_updated = chrono::Utc::now().to_rfc3339();
        // Keep existing mode if it exists, otherwise default to Player
        if existing_config.mode.is_none() {
            existing_config.mode = Some(Mode::Player);
        }

        log::info!("Updated existing config");
        existing_config
    } else {
        // Create new config
        log::info!("Creating new config");
        NerevarConfig {
            tes3mp_path: permanent_tes3mp_path.to_string_lossy().to_string(),
            version: version,
            last_updated: chrono::Utc::now().to_rfc3339(),
            mode: Some(Mode::Player), // Default to player mode
        }
    };

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Config saved to: {}", config_path.display());

    Ok(format!(
        "TES3MP installed successfully to: {}",
        permanent_tes3mp_path.display()
    ))
}

fn find_tes3mp_folder(extract_path: &Path) -> Result<std::path::PathBuf, String> {
    log::info!("Scanning directory: {}", extract_path.display());

    let entries = fs::read_dir(extract_path)
        .map_err(|e| format!("Failed to read extraction directory: {}", e))?;

    let mut found_folders = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        log::info!("Found entry: {}", path.display());

        if path.is_dir() {
            let folder_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            log::info!("Checking folder: '{}'", folder_name);
            found_folders.push(folder_name.to_string());

            // Look for folders that start with "tes3mp" (case insensitive)
            if folder_name.to_lowercase().starts_with("tes3mp") {
                log::info!("Found matching TES3MP folder: {}", path.display());
                return Ok(path);
            }
        }
    }

    log::error!(
        "No TES3MP folder found. Available folders: {:?}",
        found_folders
    );
    Err(format!(
        "No TES3MP folder found in: {}. Available folders: {:?}",
        extract_path.display(),
        found_folders
    ))
}

fn get_appdata_dir() -> Result<std::path::PathBuf, String> {
    let appdata = std::env::var("APPDATA")
        .map_err(|e| format!("Failed to get APPDATA environment variable: {}", e))?;

    let nerevar_dir = Path::new(&appdata).join("Nerevar");

    // Create the directory if it doesn't exist
    if !nerevar_dir.exists() {
        fs::create_dir_all(&nerevar_dir)
            .map_err(|e| format!("Failed to create Nerevar directory: {}", e))?;
        log::info!("Created Nerevar directory: {}", nerevar_dir.display());
    }

    Ok(nerevar_dir)
}

#[tauri::command]
fn get_nerevar_config() -> Result<Option<NerevarConfig>, String> {
    let appdata_dir = get_appdata_dir()?;
    let config_path = appdata_dir.join("config.json");

    if !config_path.exists() {
        log::info!("No config file found at: {}", config_path.display());
        return Ok(None);
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: NerevarConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    log::info!("Loaded Nerevar config: {}", config.version);
    Ok(Some(config))
}

#[tauri::command]
fn get_openmw_config() -> Result<Option<OpenMWConfig>, String> {
    // Get the Documents folder using dirs crate
    let documents_dir = get_documents_folder()?;
    let openmw_dir = documents_dir.join("My Games/OpenMW");
    let config_path = openmw_dir.join("openmw.cfg");

    log::info!("Checking for openmw.cfg at: {}", config_path.display());

    // Check if config file exists first
    if !config_path.exists() {
        log::info!("No config file found at: {}", config_path.display());
        return Ok(None);
    }

    // Read the config file content
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    log::info!("Read config file content ({} bytes)", config_content.len());

    // Parse OpenMW config file (INI format)
    let config = parse_openmw_config(&config_content)?;

    log::info!("Loaded OpenMW config with {} settings", config.len());
    Ok(Some(config))
}

fn parse_openmw_config(content: &str) -> Result<OpenMWConfig, String> {
    let mut config = std::collections::HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value pairs
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Remove quotes if present
            let clean_value = value.trim_matches('"').trim_matches('\'');

            // Try to parse as different types, defaulting to string
            let json_value = if clean_value.parse::<i64>().is_ok() {
                serde_json::Value::Number(serde_json::Number::from(
                    clean_value.parse::<i64>().unwrap(),
                ))
            } else if clean_value.parse::<f64>().is_ok() {
                serde_json::Value::Number(
                    serde_json::Number::from_f64(clean_value.parse::<f64>().unwrap()).unwrap(),
                )
            } else if clean_value == "true" || clean_value == "false" {
                serde_json::Value::Bool(clean_value == "true")
            } else {
                serde_json::Value::String(clean_value.to_string())
            };

            config.insert(key.to_string(), json_value);
        }
    }

    Ok(config)
}

#[tauri::command]
async fn run_openmw_wizard(app_handle: tauri::AppHandle) -> Result<String, String> {
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
async fn run_openmw_launcher(app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Running OpenMW launcher");

    // Get the nerevar config to find the TES3MP installation path
    let nerevar_config =
        get_nerevar_config().map_err(|e| format!("Failed to get Nerevar config: {}", e))?;

    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;

    // Construct the path to the OpenMW wizard executable
    let openmw_launcher_path = Path::new(&tes3mp_path).join("openmw-launcher.exe");

    // Check if the wizard executable exists
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

    // Spawn the OpenMW wizard process
    let mut child = std::process::Command::new(&openmw_launcher_path)
        .spawn()
        .map_err(|e| format!("Failed to run OpenMW launcher: {}", e))?;

    let pid = child.id();
    log::info!("OpenMW launcher started successfully (PID: {})", pid);

    // Send initial event that wizard started
    app_handle
        .emit("openmw-launcher-started", &pid)
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
                    log::error!("Failed to emit wizard exited event: {}", e);
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
                    log::error!("Failed to emit wizard error event: {}", emit_err);
                }
            }
        }
    });

    Ok(format!(
        "OpenMW launcher started successfully (PID: {})",
        pid
    ))
}

// remember to call `.manage(MyState::default())`
#[tauri::command]
async fn check_for_tes3mp_update() -> Result<UpdateCheckResponse, String> {
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
fn set_mode(mode: Mode) -> Result<String, String> {
    let appdata_dir = get_appdata_dir()?;
    let config_path = appdata_dir.join("config.json");

    let mut config = if config_path.exists() {
        // Load existing config
        let existing_config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read existing config: {}", e))?;

        serde_json::from_str::<NerevarConfig>(&existing_config_content)
            .map_err(|e| format!("Failed to parse existing config: {}", e))?
    } else {
        // Create default config if it doesn't exist
        log::info!("No config found, creating default config");
        NerevarConfig {
            tes3mp_path: "".to_string(), // Will be set when TES3MP is installed
            version: "0.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            mode: Some(mode),
        }
    };

    // Update only the mode field
    config.mode = Some(mode);

    // Save the updated config
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Mode updated to: {:?}", mode);
    Ok(format!("Mode set to: {:?}", mode))
}

#[tauri::command]
fn get_app_version() -> Result<String, String> {
    // Get the app version from the Tauri context
    let version = env!("CARGO_PKG_VERSION");
    Ok(version.to_string())
}

#[tauri::command]
async fn check_for_app_update() -> Result<UpdateCheckResponse, String> {
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
async fn download_app_update(download_url: String) -> Result<String, String> {
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
async fn apply_app_update(temp_file_path: String) -> Result<String, String> {
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
async fn run_tes3mp_browser(app_handle: tauri::AppHandle) -> Result<String, String> {
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
async fn run_tes3mp(app_handle: tauri::AppHandle) -> Result<String, String> {
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
async fn ping_server_tcp(ip: String, port: u16) -> Option<u128> {
    let addr = format!("{}:{}", ip, port);
    let start = Instant::now();
    match TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_secs(2)) {
        Ok(_) => Some(start.elapsed().as_millis()),
        Err(_) => None,
    }
}

#[tauri::command]
async fn set_tes3mp_client_config(ip: String, port: u16, password: String) -> Result<bool, String> {
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
    let updated_content = update_config_values(&config_content, &ip, port, &password)?;

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

fn update_config_values(
    content: &str,
    ip: &str,
    port: u16,
    password: &str,
) -> Result<String, String> {
    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut updated = false;

    // Track which section we're currently in
    let mut current_section = None;

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        // Check if this is a section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
            continue;
        }

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this line contains a key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();

            // Update values based on the current section and key
            match current_section.as_deref() {
                Some("General") => match key {
                    "destinationAddress" => {
                        *line = format!("destinationAddress = {}", ip);
                        updated = true;
                        log::info!("Updated destinationAddress to: {}", ip);
                    }
                    "port" => {
                        *line = format!("port = {}", port);
                        updated = true;
                        log::info!("Updated port to: {}", port);
                    }
                    "password" => {
                        *line = format!("password = {}", password);
                        updated = true;
                        log::info!(
                            "Updated password to: {}",
                            if password.is_empty() { "empty" } else { "set" }
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    if !updated {
        return Err("No matching configuration keys found to update".to_string());
    }

    Ok(lines.join("\n"))
}

#[derive(Serialize, Deserialize)]
struct Tes3MPServerConfig {
    general: GeneralConfig,
    plugins: PluginsConfig,
    master_server: MasterServerConfig,
}

#[derive(Serialize, Deserialize)]
struct GeneralConfig {
    local_address: String,
    port: u16,
    maximum_players: u16,
    hostname: String,
    log_level: u8,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct PluginsConfig {
    home: String,
    plugins: String,
}

#[derive(Serialize, Deserialize)]
struct MasterServerConfig {
    enabled: bool,
    address: String,
    port: u16,
    rate: u32,
}

fn parse_server_config(content: &str) -> Result<Tes3MPServerConfig, String> {
    let mut general = GeneralConfig {
        local_address: "0.0.0.0".to_string(),
        port: 25565,
        maximum_players: 64,
        hostname: "TES3MP server".to_string(),
        log_level: 1,
        password: String::new(),
    };

    let mut plugins = PluginsConfig {
        home: "./server".to_string(),
        plugins: "serverCore.lua".to_string(),
    };

    let mut master_server = MasterServerConfig {
        enabled: true,
        address: "master.tes3mp.com".to_string(),
        port: 25561,
        rate: 10000,
    };

    let mut current_section = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this is a section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
            continue;
        }

        // Check if this line contains a key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim();

            match current_section.as_deref() {
                Some("General") => match key {
                    "localAddress" => general.local_address = value.to_string(),
                    "port" => general.port = value.parse().unwrap_or(25565),
                    "maximumPlayers" => general.maximum_players = value.parse().unwrap_or(64),
                    "hostname" => general.hostname = value.to_string(),
                    "logLevel" => general.log_level = value.parse().unwrap_or(1),
                    "password" => general.password = value.to_string(),
                    _ => {}
                },
                Some("Plugins") => match key {
                    "home" => plugins.home = value.to_string(),
                    "plugins" => plugins.plugins = value.to_string(),
                    _ => {}
                },
                Some("MasterServer") => match key {
                    "enabled" => master_server.enabled = value.parse().unwrap_or(true),
                    "address" => master_server.address = value.to_string(),
                    "port" => master_server.port = value.parse().unwrap_or(25561),
                    "rate" => master_server.rate = value.parse().unwrap_or(10000),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(Tes3MPServerConfig {
        general,
        plugins,
        master_server,
    })
}

// Server Settings structures for Lua config parsing
#[derive(Serialize, Deserialize, Debug)]
struct ServerSettings {
    config: ConfigSettings,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigSettings {
    game_mode: String,
    login_time: i32,
    max_clients_per_ip: i32,
    difficulty: i32,
    game_settings: Vec<GameSetting>,
    vr_settings: Vec<VrSetting>,
    default_time_table: DefaultTimeTable,
    world_startup_scripts: Vec<String>,
    player_startup_scripts: Vec<String>,
    pass_time_when_empty: bool,
    night_start_hour: i32,
    night_end_hour: i32,
    allow_console: bool,
    allow_bed_rest: bool,
    allow_wilderness_rest: bool,
    allow_wait: bool,
    share_journal: bool,
    share_faction_ranks: bool,
    share_faction_expulsion: bool,
    share_faction_reputation: bool,
    share_topics: bool,
    share_bounty: bool,
    share_reputation: bool,
    share_map_exploration: bool,
    share_videos: bool,
    use_instanced_spawn: bool,
    instanced_spawn: SpawnLocation,
    noninstanced_spawn: SpawnLocation,
    default_respawn: RespawnLocation,
    respawn_at_imperial_shrine: bool,
    respawn_at_tribunal_temple: bool,
    forbidden_cells: Vec<String>,
    max_attribute_value: i32,
    max_speed_value: i32,
    max_skill_value: i32,
    max_acrobatics_value: i32,
    ignore_modifier_with_max_skill: bool,
    banned_equipment_items: Vec<String>,
    players_respawn: bool,
    death_time: i32,
    death_penalty_jail_days: i32,
    bounty_reset_on_death: bool,
    bounty_death_penalty: bool,
    allow_suicide_command: bool,
    allow_fixme_command: bool,
    fixme_interval: i32,
    rank_colors: RankColors,
    ping_difference_required_for_authority: i32,
    enforced_log_level: i32,
    physics_framerate: i32,
    allow_on_container_for_unloaded_cells: bool,
    enable_player_collision: bool,
    enable_actor_collision: bool,
    enable_placed_object_collision: bool,
    enforced_collision_ref_ids: Vec<String>,
    use_actor_collision_for_placed_objects: bool,
    maximum_object_scale: f64,
    enforce_data_files: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameSetting {
    name: String,
    value: serde_json::Value, // Can be bool or number
}

#[derive(Serialize, Deserialize, Debug)]
struct VrSetting {
    name: String,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DefaultTimeTable {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    days_passed: i32,
    day_time_scale: i32,
    night_time_scale: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpawnLocation {
    cell_description: String,
    position: Vec<f64>,
    rotation: Vec<f64>,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RespawnLocation {
    cell_description: String,
    position: Vec<f64>,
    rotation: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RankColors {
    server_owner: String,
    admin: String,
    moderator: String,
}

fn parse_server_settings(content: &str) -> Result<ServerSettings, String> {
    let mut config = ConfigSettings {
        game_mode: "Default".to_string(),
        login_time: 60,
        max_clients_per_ip: 3,
        difficulty: 0,
        game_settings: Vec::new(),
        vr_settings: Vec::new(),
        default_time_table: DefaultTimeTable {
            year: 427,
            month: 7,
            day: 16,
            hour: 9,
            days_passed: 1,
            day_time_scale: 30,
            night_time_scale: 40,
        },
        world_startup_scripts: Vec::new(),
        player_startup_scripts: Vec::new(),
        pass_time_when_empty: false,
        night_start_hour: 20,
        night_end_hour: 6,
        allow_console: false,
        allow_bed_rest: true,
        allow_wilderness_rest: true,
        allow_wait: true,
        share_journal: true,
        share_faction_ranks: true,
        share_faction_expulsion: false,
        share_faction_reputation: true,
        share_topics: true,
        share_bounty: false,
        share_reputation: true,
        share_map_exploration: false,
        share_videos: true,
        use_instanced_spawn: true,
        instanced_spawn: SpawnLocation {
            cell_description: "Seyda Neen, Census and Excise Office".to_string(),
            position: vec![1130.3388671875, -387.14947509766, 193.0],
            rotation: vec![0.09375, 1.5078122615814],
            text: "Multiplayer skips several minutes of the game's introduction and places you at the first quest giver.".to_string(),
        },
        noninstanced_spawn: SpawnLocation {
            cell_description: "-3, -2".to_string(),
            position: vec![-23894.0, -15079.0, 505.0],
            rotation: vec![0.0, 1.2],
            text: "Multiplayer skips over the original character generation.".to_string(),
        },
        default_respawn: RespawnLocation {
            cell_description: "Balmora, Temple".to_string(),
            position: vec![4700.5673828125, 3874.7416992188, 14758.990234375],
            rotation: vec![0.25314688682556, 1.570611000061],
        },
        respawn_at_imperial_shrine: true,
        respawn_at_tribunal_temple: true,
        forbidden_cells: Vec::new(),
        max_attribute_value: 200,
        max_speed_value: 365,
        max_skill_value: 200,
        max_acrobatics_value: 1200,
        ignore_modifier_with_max_skill: false,
        banned_equipment_items: Vec::new(),
        players_respawn: true,
        death_time: 5,
        death_penalty_jail_days: 5,
        bounty_reset_on_death: false,
        bounty_death_penalty: false,
        allow_suicide_command: true,
        allow_fixme_command: true,
        fixme_interval: 30,
        rank_colors: RankColors {
            server_owner: "Orange".to_string(),
            admin: "Red".to_string(),
            moderator: "Green".to_string(),
        },
        ping_difference_required_for_authority: 40,
        enforced_log_level: -1,
        physics_framerate: 60,
        allow_on_container_for_unloaded_cells: false,
        enable_player_collision: true,
        enable_actor_collision: true,
        enable_placed_object_collision: false,
        enforced_collision_ref_ids: Vec::new(),
        use_actor_collision_for_placed_objects: false,
        maximum_object_scale: 2.0,
        enforce_data_files: false,
    };

    let mut lines = content.lines().peekable();
    let mut in_table = false;
    let mut current_table_name = String::new();
    let mut table_content = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        // Handle table definitions
        if trimmed.starts_with("config.") && trimmed.contains("=") {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[6..eq_pos].trim(); // Skip "config."
                let value = trimmed[eq_pos + 1..].trim();

                // Check if this starts a table
                if value.starts_with('{') {
                    in_table = true;
                    current_table_name = key.to_string();
                    table_content.clear();

                    // Handle single-line tables
                    if value.ends_with('}') {
                        parse_table_content(
                            &value[1..value.len() - 1],
                            &current_table_name,
                            &mut config,
                        )?;
                        in_table = false;
                    } else {
                        // Multi-line table
                        let content = value[1..].trim();
                        if !content.is_empty() {
                            table_content.push(content);
                        }
                    }
                } else {
                    // Simple key-value pair
                    parse_simple_value(key, value, &mut config)?;
                }
            }
        } else if in_table {
            // We're inside a table
            if trimmed == "}" {
                // End of table
                let content = table_content.join(" ");
                parse_table_content(&content, &current_table_name, &mut config)?;
                in_table = false;
                table_content.clear();
            } else {
                // Add line to table content
                table_content.push(trimmed);
            }
        }
    }

    Ok(ServerSettings { config })
}

fn parse_simple_value(key: &str, value: &str, config: &mut ConfigSettings) -> Result<(), String> {
    match key {
        "gameMode" => {
            config.game_mode = parse_string_value(value)?;
        }
        "loginTime" => {
            config.login_time = parse_number_value(value)?;
        }
        "maxClientsPerIP" => {
            config.max_clients_per_ip = parse_number_value(value)?;
        }
        "difficulty" => {
            config.difficulty = parse_number_value(value)?;
        }
        "passTimeWhenEmpty" => {
            config.pass_time_when_empty = parse_bool_value(value)?;
        }
        "nightStartHour" => {
            config.night_start_hour = parse_number_value(value)?;
        }
        "nightEndHour" => {
            config.night_end_hour = parse_number_value(value)?;
        }
        "allowConsole" => {
            config.allow_console = parse_bool_value(value)?;
        }
        "allowBedRest" => {
            config.allow_bed_rest = parse_bool_value(value)?;
        }
        "allowWildernessRest" => {
            config.allow_wilderness_rest = parse_bool_value(value)?;
        }
        "allowWait" => {
            config.allow_wait = parse_bool_value(value)?;
        }
        "shareJournal" => {
            config.share_journal = parse_bool_value(value)?;
        }
        "shareFactionRanks" => {
            config.share_faction_ranks = parse_bool_value(value)?;
        }
        "shareFactionExpulsion" => {
            config.share_faction_expulsion = parse_bool_value(value)?;
        }
        "shareFactionReputation" => {
            config.share_faction_reputation = parse_bool_value(value)?;
        }
        "shareTopics" => {
            config.share_topics = parse_bool_value(value)?;
        }
        "shareBounty" => {
            config.share_bounty = parse_bool_value(value)?;
        }
        "shareReputation" => {
            config.share_reputation = parse_bool_value(value)?;
        }
        "shareMapExploration" => {
            config.share_map_exploration = parse_bool_value(value)?;
        }
        "shareVideos" => {
            config.share_videos = parse_bool_value(value)?;
        }
        "useInstancedSpawn" => {
            config.use_instanced_spawn = parse_bool_value(value)?;
        }
        "respawnAtImperialShrine" => {
            config.respawn_at_imperial_shrine = parse_bool_value(value)?;
        }
        "respawnAtTribunalTemple" => {
            config.respawn_at_tribunal_temple = parse_bool_value(value)?;
        }
        "maxAttributeValue" => {
            config.max_attribute_value = parse_number_value(value)?;
        }
        "maxSpeedValue" => {
            config.max_speed_value = parse_number_value(value)?;
        }
        "maxSkillValue" => {
            config.max_skill_value = parse_number_value(value)?;
        }
        "maxAcrobaticsValue" => {
            config.max_acrobatics_value = parse_number_value(value)?;
        }
        "ignoreModifierWithMaxSkill" => {
            config.ignore_modifier_with_max_skill = parse_bool_value(value)?;
        }
        "playersRespawn" => {
            config.players_respawn = parse_bool_value(value)?;
        }
        "deathTime" => {
            config.death_time = parse_number_value(value)?;
        }
        "deathPenaltyJailDays" => {
            config.death_penalty_jail_days = parse_number_value(value)?;
        }
        "bountyResetOnDeath" => {
            config.bounty_reset_on_death = parse_bool_value(value)?;
        }
        "bountyDeathPenalty" => {
            config.bounty_death_penalty = parse_bool_value(value)?;
        }
        "allowSuicideCommand" => {
            config.allow_suicide_command = parse_bool_value(value)?;
        }
        "allowFixmeCommand" => {
            config.allow_fixme_command = parse_bool_value(value)?;
        }
        "fixmeInterval" => {
            config.fixme_interval = parse_number_value(value)?;
        }
        "pingDifferenceRequiredForAuthority" => {
            config.ping_difference_required_for_authority = parse_number_value(value)?;
        }
        "enforcedLogLevel" => {
            config.enforced_log_level = parse_number_value(value)?;
        }
        "physicsFramerate" => {
            config.physics_framerate = parse_number_value(value)?;
        }
        "allowOnContainerForUnloadedCells" => {
            config.allow_on_container_for_unloaded_cells = parse_bool_value(value)?;
        }
        "enablePlayerCollision" => {
            config.enable_player_collision = parse_bool_value(value)?;
        }
        "enableActorCollision" => {
            config.enable_actor_collision = parse_bool_value(value)?;
        }
        "enablePlacedObjectCollision" => {
            config.enable_placed_object_collision = parse_bool_value(value)?;
        }
        "useActorCollisionForPlacedObjects" => {
            config.use_actor_collision_for_placed_objects = parse_bool_value(value)?;
        }
        "maximumObjectScale" => {
            config.maximum_object_scale = parse_float_value(value)?;
        }
        "enforceDataFiles" => {
            config.enforce_data_files = parse_bool_value(value)?;
        }
        _ => {
            // Unknown key, skip
        }
    }
    Ok(())
}

fn parse_table_content(
    content: &str,
    table_name: &str,
    config: &mut ConfigSettings,
) -> Result<(), String> {
    match table_name {
        "gameSettings" => {
            config.game_settings = parse_game_settings_table(content)?;
        }
        "vrSettings" => {
            config.vr_settings = parse_vr_settings_table(content)?;
        }
        "defaultTimeTable" => {
            config.default_time_table = parse_time_table(content)?;
        }
        "worldStartupScripts" => {
            config.world_startup_scripts = parse_string_array(content)?;
        }
        "playerStartupScripts" => {
            config.player_startup_scripts = parse_string_array(content)?;
        }
        "forbiddenCells" => {
            config.forbidden_cells = parse_string_array(content)?;
        }
        "bannedEquipmentItems" => {
            config.banned_equipment_items = parse_string_array(content)?;
        }
        "enforcedCollisionRefIds" => {
            config.enforced_collision_ref_ids = parse_string_array(content)?;
        }
        "instancedSpawn" => {
            config.instanced_spawn = parse_spawn_location(content)?;
        }
        "noninstancedSpawn" => {
            config.noninstanced_spawn = parse_spawn_location(content)?;
        }
        "defaultRespawn" => {
            config.default_respawn = parse_respawn_location(content)?;
        }
        "rankColors" => {
            config.rank_colors = parse_rank_colors(content)?;
        }
        _ => {
            // Unknown table, skip
        }
    }
    Ok(())
}

fn parse_string_value(value: &str) -> Result<String, String> {
    if value.starts_with('"') && value.ends_with('"') {
        Ok(value[1..value.len() - 1].to_string())
    } else {
        Ok(value.to_string())
    }
}

fn parse_number_value(value: &str) -> Result<i32, String> {
    value
        .parse::<i32>()
        .map_err(|e| format!("Failed to parse number '{}': {}", value, e))
}

fn parse_float_value(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse float '{}': {}", value, e))
}

fn parse_bool_value(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("Invalid boolean value: {}", value)),
    }
}

fn parse_game_settings_table(content: &str) -> Result<Vec<GameSetting>, String> {
    let mut settings = Vec::new();
    let entries: Vec<&str> = content.split('}').collect();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() || !entry.starts_with('{') {
            continue;
        }

        let content = entry[1..].trim();
        let parts: Vec<&str> = content.split(',').collect();

        if parts.len() >= 2 {
            let name_part = parts[0].trim();
            let value_part = parts[1].trim();

            if name_part.starts_with("name = \"") && name_part.ends_with('"') {
                let name = name_part[8..name_part.len() - 1].to_string();

                let value = if value_part.starts_with("value = ") {
                    let val_str = &value_part[8..];
                    if val_str == "true" {
                        serde_json::Value::Bool(true)
                    } else if val_str == "false" {
                        serde_json::Value::Bool(false)
                    } else if let Ok(num) = val_str.parse::<i32>() {
                        serde_json::Value::Number(num.into())
                    } else {
                        serde_json::Value::String(val_str.to_string())
                    }
                } else {
                    serde_json::Value::String(value_part.to_string())
                };

                settings.push(GameSetting { name, value });
            }
        }
    }

    Ok(settings)
}

fn parse_vr_settings_table(content: &str) -> Result<Vec<VrSetting>, String> {
    let mut settings = Vec::new();
    let entries: Vec<&str> = content.split('}').collect();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() || !entry.starts_with('{') {
            continue;
        }

        let content = entry[1..].trim();
        let parts: Vec<&str> = content.split(',').collect();

        if parts.len() >= 2 {
            let name_part = parts[0].trim();
            let value_part = parts[1].trim();

            if name_part.starts_with("name = \"") && name_part.ends_with('"') {
                let name = name_part[8..name_part.len() - 1].to_string();

                if value_part.starts_with("value = ") {
                    let val_str = &value_part[8..];
                    if let Ok(value) = val_str.parse::<f64>() {
                        settings.push(VrSetting { name, value });
                    }
                }
            }
        }
    }

    Ok(settings)
}

fn parse_time_table(content: &str) -> Result<DefaultTimeTable, String> {
    let mut time_table = DefaultTimeTable {
        year: 427,
        month: 7,
        day: 16,
        hour: 9,
        days_passed: 1,
        day_time_scale: 30,
        night_time_scale: 40,
    };

    let parts: Vec<&str> = content.split(',').collect();
    for part in parts {
        let part = part.trim();
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim();
            let value = part[eq_pos + 1..].trim();

            match key {
                "year" => time_table.year = parse_number_value(value)?,
                "month" => time_table.month = parse_number_value(value)?,
                "day" => time_table.day = parse_number_value(value)?,
                "hour" => time_table.hour = parse_number_value(value)?,
                "daysPassed" => time_table.days_passed = parse_number_value(value)?,
                "dayTimeScale" => time_table.day_time_scale = parse_number_value(value)?,
                "nightTimeScale" => time_table.night_time_scale = parse_number_value(value)?,
                _ => {}
            }
        }
    }

    Ok(time_table)
}

fn parse_string_array(content: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let content = content.trim();

    if content.starts_with('{') && content.ends_with('}') {
        let inner = &content[1..content.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();

        for part in parts {
            let part = part.trim();
            if part.starts_with('"') && part.ends_with('"') {
                items.push(part[1..part.len() - 1].to_string());
            } else {
                items.push(part.to_string());
            }
        }
    }

    Ok(items)
}

fn parse_spawn_location(content: &str) -> Result<SpawnLocation, String> {
    let mut spawn = SpawnLocation {
        cell_description: String::new(),
        position: Vec::new(),
        rotation: Vec::new(),
        text: String::new(),
    };

    let parts: Vec<&str> = content.split(',').collect();
    for part in parts {
        let part = part.trim();
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim();
            let value = part[eq_pos + 1..].trim();

            match key {
                "cellDescription" => {
                    spawn.cell_description = parse_string_value(value)?;
                }
                "position" => {
                    if value.starts_with('{') && value.ends_with('}') {
                        let inner = &value[1..value.len() - 1];
                        let coords: Vec<&str> = inner.split(',').collect();
                        spawn.position = coords
                            .iter()
                            .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
                            .collect();
                    }
                }
                "rotation" => {
                    if value.starts_with('{') && value.ends_with('}') {
                        let inner = &value[1..value.len() - 1];
                        let coords: Vec<&str> = inner.split(',').collect();
                        spawn.rotation = coords
                            .iter()
                            .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
                            .collect();
                    }
                }
                "text" => {
                    spawn.text = parse_string_value(value)?;
                }
                _ => {}
            }
        }
    }

    Ok(spawn)
}

fn parse_respawn_location(content: &str) -> Result<RespawnLocation, String> {
    let mut respawn = RespawnLocation {
        cell_description: String::new(),
        position: Vec::new(),
        rotation: Vec::new(),
    };

    let parts: Vec<&str> = content.split(',').collect();
    for part in parts {
        let part = part.trim();
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim();
            let value = part[eq_pos + 1..].trim();

            match key {
                "cellDescription" => {
                    respawn.cell_description = parse_string_value(value)?;
                }
                "position" => {
                    if value.starts_with('{') && value.ends_with('}') {
                        let inner = &value[1..value.len() - 1];
                        let coords: Vec<&str> = inner.split(',').collect();
                        respawn.position = coords
                            .iter()
                            .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
                            .collect();
                    }
                }
                "rotation" => {
                    if value.starts_with('{') && value.ends_with('}') {
                        let inner = &value[1..value.len() - 1];
                        let coords: Vec<&str> = inner.split(',').collect();
                        respawn.rotation = coords
                            .iter()
                            .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
                            .collect();
                    }
                }
                _ => {}
            }
        }
    }

    Ok(respawn)
}

fn parse_rank_colors(content: &str) -> Result<RankColors, String> {
    let mut colors = RankColors {
        server_owner: "Orange".to_string(),
        admin: "Red".to_string(),
        moderator: "Green".to_string(),
    };

    let parts: Vec<&str> = content.split(',').collect();
    for part in parts {
        let part = part.trim();
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim();
            let value = part[eq_pos + 1..].trim();

            match key {
                "serverOwner" => {
                    colors.server_owner = parse_string_value(value)?;
                }
                "admin" => {
                    colors.admin = parse_string_value(value)?;
                }
                "moderator" => {
                    colors.moderator = parse_string_value(value)?;
                }
                _ => {}
            }
        }
    }

    Ok(colors)
}

fn write_server_settings(settings: &ServerSettings) -> Result<String, String> {
    let mut content = String::new();

    // Header
    content.push_str("config = {}\n\n");

    // Basic settings
    content.push_str(&format!(
        "-- The path used by the server for its data folder\n"
    ));
    content.push_str(&format!("config.dataPath = tes3mp.GetDataPath()\n\n"));

    content.push_str(&format!(
        "-- The game mode displayed for this server in the server browser\n"
    ));
    content.push_str(&format!(
        "config.gameMode = \"{}\"\n\n",
        settings.config.game_mode
    ));

    content.push_str(&format!("-- Time to login, in seconds\n"));
    content.push_str(&format!(
        "config.loginTime = {}\n\n",
        settings.config.login_time
    ));

    content.push_str(&format!(
        "-- How many clients are allowed to connect from the same IP address\n"
    ));
    content.push_str(&format!(
        "config.maxClientsPerIP = {}\n\n",
        settings.config.max_clients_per_ip
    ));

    content.push_str(&format!("-- The difficulty level used by default\n"));
    content.push_str(&format!("-- Note: In OpenMW, the difficulty slider goes between -100 and 100, with 0 as the default,\n"));
    content.push_str(&format!(
        "--       though you can use any integer value here\n"
    ));
    content.push_str(&format!(
        "config.difficulty = {}\n\n",
        settings.config.difficulty
    ));

    // Game settings table
    content.push_str(&format!("-- The game settings to enforce for players\n"));
    content.push_str(&format!("-- Note 1: Anything from OpenMW's game settings can be added here, which means anything listed\n"));
    content.push_str(&format!("--         on https://openmw.readthedocs.io/en/latest/reference/modding/settings/game.html\n"));
    content.push_str(&format!("-- Note 2: Some settings, such as \"difficulty\" and \"actors processing range\", cannot be\n"));
    content.push_str(&format!("--         changed from here\n"));
    content.push_str(&format!("config.gameSettings = {{\n"));
    for setting in &settings.config.game_settings {
        let value_str = match &setting.value {
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => format!("\"{}\"", s),
            _ => "nil".to_string(),
        };
        content.push_str(&format!(
            "    {{ name = \"{}\", value = {} }},\n",
            setting.name, value_str
        ));
    }
    content.push_str(&format!("}}\n\n"));

    // VR settings table
    content.push_str(&format!("-- The VR settings to enforce for players\n"));
    content.push_str(&format!("config.vrSettings = {{\n"));
    for setting in &settings.config.vr_settings {
        content.push_str(&format!(
            "    {{ name = \"{}\", value = {} }},\n",
            setting.name, setting.value
        ));
    }
    content.push_str(&format!("}}\n\n"));

    // Default time table
    content.push_str(&format!(
        "-- The world time used for a newly created world\n"
    ));
    content.push_str(&format!(
        "config.defaultTimeTable = {{ year = {}, month = {}, day = {}, hour = {},\n",
        settings.config.default_time_table.year,
        settings.config.default_time_table.month,
        settings.config.default_time_table.day,
        settings.config.default_time_table.hour
    ));
    content.push_str(&format!(
        "    daysPassed = {}, dayTimeScale = {}, nightTimeScale = {} }}\n\n",
        settings.config.default_time_table.days_passed,
        settings.config.default_time_table.day_time_scale,
        settings.config.default_time_table.night_time_scale
    ));

    // World startup scripts
    content.push_str(&format!(
        "-- Which ingame startup scripts should be run via the /runstartup command\n"
    ));
    content.push_str(&format!(
        "-- Note: These affect the world and must not be run for every player who joins.\n"
    ));
    content.push_str(&format!("config.worldStartupScripts = {{"));
    for (i, script) in settings.config.world_startup_scripts.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("\"{}\"", script));
    }
    content.push_str(&format!("}}\n\n"));

    // Player startup scripts
    content.push_str(&format!(
        "-- Which ingame startup scripts should be run on every player who joins\n"
    ));
    content.push_str(&format!("-- Note: These pertain to game mechanics that wouldn't work otherwise, such as vampirism checks\n"));
    content.push_str(&format!("config.playerStartupScripts = {{"));
    for (i, script) in settings.config.player_startup_scripts.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("\"{}\"", script));
    }
    content.push_str(&format!("}}\n\n"));

    // Boolean settings
    content.push_str(&format!("-- Whether the world time should continue passing when there are no players on the server\n"));
    content.push_str(&format!(
        "config.passTimeWhenEmpty = {}\n\n",
        settings.config.pass_time_when_empty
    ));

    content.push_str(&format!("-- The hours at which night is regarded as starting and ending, used to pass time using a\n"));
    content.push_str(&format!("-- different timescale when it's night\n"));
    content.push_str(&format!(
        "config.nightStartHour = {}\n",
        settings.config.night_start_hour
    ));
    content.push_str(&format!(
        "config.nightEndHour = {}\n\n",
        settings.config.night_end_hour
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to use the ingame tilde (~) console by default\n"
    ));
    content.push_str(&format!(
        "config.allowConsole = {}\n\n",
        settings.config.allow_console
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to rest in bed by default\n"
    ));
    content.push_str(&format!(
        "config.allowBedRest = {}\n\n",
        settings.config.allow_bed_rest
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to rest in the wilderness by default\n"
    ));
    content.push_str(&format!(
        "config.allowWildernessRest = {}\n\n",
        settings.config.allow_wilderness_rest
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to wait by default\n"
    ));
    content.push_str(&format!(
        "config.allowWait = {}\n\n",
        settings.config.allow_wait
    ));

    content.push_str(&format!(
        "-- Whether journal entries should be shared across the players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareJournal = {}\n\n",
        settings.config.share_journal
    ));

    content.push_str(&format!(
        "-- Whether faction ranks should be shared across the players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareFactionRanks = {}\n\n",
        settings.config.share_faction_ranks
    ));

    content.push_str(&format!(
        "-- Whether faction expulsion should be shared across the players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareFactionExpulsion = {}\n\n",
        settings.config.share_faction_expulsion
    ));

    content.push_str(&format!(
        "-- Whether faction reputation should be shared across the players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareFactionReputation = {}\n\n",
        settings.config.share_faction_reputation
    ));

    content.push_str(&format!(
        "-- Whether dialogue topics should be shared across the players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareTopics = {}\n\n",
        settings.config.share_topics
    ));

    content.push_str(&format!(
        "-- Whether crime bounties should be shared across players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareBounty = {}\n\n",
        settings.config.share_bounty
    ));

    content.push_str(&format!(
        "-- Whether reputation should be shared across players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareReputation = {}\n\n",
        settings.config.share_reputation
    ));

    content.push_str(&format!(
        "-- Whether map exploration should be shared across players on the server or not\n"
    ));
    content.push_str(&format!(
        "config.shareMapExploration = {}\n\n",
        settings.config.share_map_exploration
    ));

    content.push_str(&format!("-- Whether ingame videos should be played for other players when triggered by one player\n"));
    content.push_str(&format!(
        "config.shareVideos = {}\n\n",
        settings.config.share_videos
    ));

    content.push_str(&format!(
        "-- Whether the instanced spawn should be used instead of the noninstanced one\n"
    ));
    content.push_str(&format!(
        "config.useInstancedSpawn = {}\n\n",
        settings.config.use_instanced_spawn
    ));

    // Spawn locations
    content.push_str(&format!("-- Where players will be spawned if an instanced spawn is desired, with a different clean copy of\n"));
    content.push_str(&format!("-- this cell existing for each player\n"));
    content.push_str(&format!(
        "-- Warning: Only interior cells can be instanced\n"
    ));
    content.push_str(&format!("config.instancedSpawn = {{\n"));
    content.push_str(&format!(
        "    cellDescription = \"{}\",\n",
        settings.config.instanced_spawn.cell_description
    ));
    content.push_str(&format!("    position = {{"));
    for (i, pos) in settings.config.instanced_spawn.position.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", pos));
    }
    content.push_str(&format!("}},\n"));
    content.push_str(&format!("    rotation = {{"));
    for (i, rot) in settings.config.instanced_spawn.rotation.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", rot));
    }
    content.push_str(&format!("}},\n"));
    content.push_str(&format!(
        "    text = \"{}\"\n",
        settings.config.instanced_spawn.text
    ));
    content.push_str(&format!("}}\n\n"));

    content.push_str(&format!(
        "-- Where players will be spawned if an instanced spawn is not desired\n"
    ));
    content.push_str(&format!("config.noninstancedSpawn = {{\n"));
    content.push_str(&format!(
        "    cellDescription = \"{}\",\n",
        settings.config.noninstanced_spawn.cell_description
    ));
    content.push_str(&format!("    position = {{"));
    for (i, pos) in settings
        .config
        .noninstanced_spawn
        .position
        .iter()
        .enumerate()
    {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", pos));
    }
    content.push_str(&format!("}},\n"));
    content.push_str(&format!("    rotation = {{"));
    for (i, rot) in settings
        .config
        .noninstanced_spawn
        .rotation
        .iter()
        .enumerate()
    {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", rot));
    }
    content.push_str(&format!("}},\n"));
    content.push_str(&format!(
        "    text = \"{}\"\n",
        settings.config.noninstanced_spawn.text
    ));
    content.push_str(&format!("}}\n\n"));

    // Default respawn
    content.push_str(&format!("-- The location that players respawn at, unless overridden below by other respawn options\n"));
    content.push_str(&format!("config.defaultRespawn = {{\n"));
    content.push_str(&format!(
        "    cellDescription = \"{}\",\n",
        settings.config.default_respawn.cell_description
    ));
    content.push_str(&format!("    position = {{"));
    for (i, pos) in settings.config.default_respawn.position.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", pos));
    }
    content.push_str(&format!("}},\n"));
    content.push_str(&format!("    rotation = {{"));
    for (i, rot) in settings.config.default_respawn.rotation.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("{}", rot));
    }
    content.push_str(&format!("}}\n"));
    content.push_str(&format!("}}\n\n"));

    // Respawn options
    content.push_str(&format!(
        "-- Whether the default respawn location should be ignored in favor of respawning the\n"
    ));
    content.push_str(&format!("-- player at the nearest Imperial shrine\n"));
    content.push_str(&format!(
        "config.respawnAtImperialShrine = {}\n\n",
        settings.config.respawn_at_imperial_shrine
    ));

    content.push_str(&format!(
        "-- Whether the default respawn location should be ignored in favor of respawning the\n"
    ));
    content.push_str(&format!("-- player at the nearest Tribunal temple\n"));
    content.push_str(&format!(
        "-- Note: When both this and the Imperial shrine option are enabled, there is a 50%\n"
    ));
    content.push_str(&format!(
        "--       chance of the player being respawned at either\n"
    ));
    content.push_str(&format!(
        "config.respawnAtTribunalTemple = {}\n\n",
        settings.config.respawn_at_tribunal_temple
    ));

    // Forbidden cells
    content.push_str(&format!(
        "-- The cells that players are forbidden from entering, with any attempt to enter them\n"
    ));
    content.push_str(&format!(
        "-- transporting them to the last location in their previous cell\n"
    ));
    content.push_str(&format!("config.forbiddenCells = {{"));
    for (i, cell) in settings.config.forbidden_cells.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("\"{}\"", cell));
    }
    content.push_str(&format!("}}\n\n"));

    // Max values
    content.push_str(&format!(
        "-- The maximum value that any attribute except Speed is allowed to have\n"
    ));
    content.push_str(&format!(
        "config.maxAttributeValue = {}\n\n",
        settings.config.max_attribute_value
    ));

    content.push_str(&format!(
        "-- The maximum value that Speed is allowed to have\n"
    ));
    content.push_str(&format!(
        "-- Note: Speed is given special treatment because of the Boots of Blinding Speed\n"
    ));
    content.push_str(&format!(
        "config.maxSpeedValue = {}\n\n",
        settings.config.max_speed_value
    ));

    content.push_str(&format!(
        "-- The maximum value that any skill except Acrobatics is allowed to have\n"
    ));
    content.push_str(&format!(
        "config.maxSkillValue = {}\n\n",
        settings.config.max_skill_value
    ));

    content.push_str(&format!(
        "-- The maximum value that Acrobatics is allowed to have\n"
    ));
    content.push_str(&format!(
        "-- Note: Acrobatics is given special treatment because of the Scroll of Icarian Flight\n"
    ));
    content.push_str(&format!(
        "config.maxAcrobaticsValue = {}\n\n",
        settings.config.max_acrobatics_value
    ));

    content.push_str(&format!(
        "-- Allow modifier values to bypass allowed skill values\n"
    ));
    content.push_str(&format!(
        "config.ignoreModifierWithMaxSkill = {}\n\n",
        settings.config.ignore_modifier_with_max_skill
    ));

    // Banned equipment
    content.push_str(&format!(
        "-- The refIds of items that players are not allowed to equip for balancing reasons\n"
    ));
    content.push_str(&format!("config.bannedEquipmentItems = {{"));
    for (i, item) in settings.config.banned_equipment_items.iter().enumerate() {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("\"{}\"", item));
    }
    content.push_str(&format!("}}\n\n"));

    // Death and respawn settings
    content.push_str(&format!("-- Whether players should respawn when dying\n"));
    content.push_str(&format!(
        "config.playersRespawn = {}\n\n",
        settings.config.players_respawn
    ));

    content.push_str(&format!(
        "-- Time to stay dead before being respawned, in seconds\n"
    ));
    content.push_str(&format!(
        "config.deathTime = {}\n\n",
        settings.config.death_time
    ));

    content.push_str(&format!(
        "-- The number of days spent in jail as a penalty for dying, when respawning\n"
    ));
    content.push_str(&format!(
        "config.deathPenaltyJailDays = {}\n\n",
        settings.config.death_penalty_jail_days
    ));

    content.push_str(&format!(
        "-- Whether players' bounties are reset to 0 after dying\n"
    ));
    content.push_str(&format!(
        "config.bountyResetOnDeath = {}\n\n",
        settings.config.bounty_reset_on_death
    ));

    content.push_str(&format!(
        "-- Whether players spend time in jail proportional to their bounty after dying\n"
    ));
    content.push_str(&format!(
        "-- Note: If deathPenaltyJailDays is also enabled, that penalty will be added to\n"
    ));
    content.push_str(&format!("--       this one\n"));
    content.push_str(&format!(
        "config.bountyDeathPenalty = {}\n\n",
        settings.config.bounty_death_penalty
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to use the /suicide command\n"
    ));
    content.push_str(&format!(
        "config.allowSuicideCommand = {}\n\n",
        settings.config.allow_suicide_command
    ));

    content.push_str(&format!(
        "-- Whether players should be allowed to use the /fixme command\n"
    ));
    content.push_str(&format!(
        "config.allowFixmeCommand = {}\n\n",
        settings.config.allow_fixme_command
    ));

    content.push_str(&format!(
        "-- How many seconds need to pass between uses of the /fixme command by a player\n"
    ));
    content.push_str(&format!(
        "config.fixmeInterval = {}\n\n",
        settings.config.fixme_interval
    ));

    // Rank colors
    content.push_str(&format!(
        "-- The colors used for different ranks on the server\n"
    ));
    content.push_str(&format!("config.rankColors = {{ serverOwner = color.{}, admin = color.{}, moderator = color.{} }}\n\n", 
        settings.config.rank_colors.server_owner,
        settings.config.rank_colors.admin,
        settings.config.rank_colors.moderator));

    // Authority and performance settings
    content.push_str(&format!("-- What the difference in ping needs to be in favor of a new arrival to a cell or region\n"));
    content.push_str(&format!("-- compared to that cell or region's current player authority for the new arrival to become\n"));
    content.push_str(&format!("-- the authority there\n"));
    content.push_str(&format!("-- Note: Setting this too low will lead to constant authority changes which cause more lag\n"));
    content.push_str(&format!(
        "config.pingDifferenceRequiredForAuthority = {}\n\n",
        settings.config.ping_difference_required_for_authority
    ));

    content.push_str(&format!(
        "-- The log level enforced on clients by default, determining how much debug information\n"
    ));
    content.push_str(&format!("-- is displayed in their debug window and logs\n"));
    content.push_str(&format!(
        "-- Note 1: Set this to -1 to allow clients to use whatever log level they have set in\n"
    ));
    content.push_str(&format!("--         their client settings\n"));
    content.push_str(&format!(
        "-- Note 2: If you set this to 0 or 1, clients will be able to read about the movements\n"
    ));
    content.push_str(&format!(
        "--         and actions of other players that they would otherwise not know about,\n"
    ));
    content.push_str(&format!(
        "--         while also incurring a framerate loss on highly populated servers\n"
    ));
    content.push_str(&format!(
        "config.enforcedLogLevel = {}\n\n",
        settings.config.enforced_log_level
    ));

    content.push_str(&format!("-- The physics framerate used by default\n"));
    content.push_str(&format!(
        "-- Note: In OpenMW, the physics framerate is 60 by default\n"
    ));
    content.push_str(&format!(
        "config.physicsFramerate = {}\n\n",
        settings.config.physics_framerate
    ));

    // Container and collision settings
    content.push_str(&format!(
        "-- Whether players are allowed to interact with containers located in unloaded cells.\n"
    ));
    content.push_str(&format!(
        "config.allowOnContainerForUnloadedCells = {}\n\n",
        settings.config.allow_on_container_for_unloaded_cells
    ));

    content.push_str(&format!(
        "-- Whether players should collide with other actors\n"
    ));
    content.push_str(&format!(
        "config.enablePlayerCollision = {}\n\n",
        settings.config.enable_player_collision
    ));

    content.push_str(&format!(
        "-- Whether actors should collide with other actors\n"
    ));
    content.push_str(&format!(
        "config.enableActorCollision = {}\n\n",
        settings.config.enable_actor_collision
    ));

    content.push_str(&format!(
        "-- Whether placed objects should collide with actors\n"
    ));
    content.push_str(&format!(
        "config.enablePlacedObjectCollision = {}\n\n",
        settings.config.enable_placed_object_collision
    ));

    // Enforced collision ref IDs
    content.push_str(&format!("-- Enforce collision for certain placed object refIds even when enablePlacedObjectCollision\n"));
    content.push_str(&format!("-- is false\n"));
    content.push_str(&format!("config.enforcedCollisionRefIds = {{"));
    for (i, ref_id) in settings
        .config
        .enforced_collision_ref_ids
        .iter()
        .enumerate()
    {
        if i > 0 {
            content.push_str(", ");
        }
        content.push_str(&format!("\"{}\"", ref_id));
    }
    content.push_str(&format!("}}\n\n"));

    content.push_str(&format!("-- Whether placed object collision (when turned on) resembles actor collision, in that it\n"));
    content.push_str(&format!(
        "-- prevents players from standing on top of the placed objects without slipping\n"
    ));
    content.push_str(&format!(
        "config.useActorCollisionForPlacedObjects = {}\n\n",
        settings.config.use_actor_collision_for_placed_objects
    ));

    content.push_str(&format!(
        "-- The maximum scale that objects are allowed to have\n"
    ));
    content.push_str(&format!(
        "config.maximumObjectScale = {}\n\n",
        settings.config.maximum_object_scale
    ));

    content.push_str(&format!("-- Whether data files should be enforced\n"));
    content.push_str(&format!(
        "config.enforceDataFiles = {}\n",
        settings.config.enforce_data_files
    ));

    Ok(content)
}

fn update_server_config_values(
    content: &str,
    config: &serde_json::Value,
) -> Result<String, String> {
    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut updated = false;

    // Track which section we're currently in
    let mut current_section = None;

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        // Check if this is a section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
            continue;
        }

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this line contains a key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();

            // Update values based on the current section and key
            match current_section.as_deref() {
                Some("General") => {
                    if let Some(general) = config.get("general") {
                        match key {
                            "localAddress" => {
                                if let Some(value) = general.get("localAddress") {
                                    if let Some(addr) = value.as_str() {
                                        *line = format!("localAddress = {}", addr);
                                        updated = true;
                                    }
                                }
                            }
                            "port" => {
                                if let Some(value) = general.get("port") {
                                    if let Some(port) = value.as_u64() {
                                        *line = format!("port = {}", port);
                                        updated = true;
                                    }
                                }
                            }
                            "maximumPlayers" => {
                                if let Some(value) = general.get("maximumPlayers") {
                                    if let Some(max_players) = value.as_u64() {
                                        *line = format!("maximumPlayers = {}", max_players);
                                        updated = true;
                                    }
                                }
                            }
                            "hostname" => {
                                if let Some(value) = general.get("hostname") {
                                    if let Some(hostname) = value.as_str() {
                                        *line = format!("hostname = {}", hostname);
                                        updated = true;
                                    }
                                }
                            }
                            "logLevel" => {
                                if let Some(value) = general.get("logLevel") {
                                    if let Some(log_level) = value.as_u64() {
                                        *line = format!("logLevel = {}", log_level);
                                        updated = true;
                                    }
                                }
                            }
                            "password" => {
                                if let Some(value) = general.get("password") {
                                    if let Some(password) = value.as_str() {
                                        *line = format!("password = {}", password);
                                        updated = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some("Plugins") => {
                    if let Some(plugins) = config.get("plugins") {
                        match key {
                            "home" => {
                                if let Some(value) = plugins.get("home") {
                                    if let Some(home) = value.as_str() {
                                        *line = format!("home = {}", home);
                                        updated = true;
                                    }
                                }
                            }
                            "plugins" => {
                                if let Some(value) = plugins.get("plugins") {
                                    if let Some(plugins_str) = value.as_str() {
                                        *line = format!("plugins = {}", plugins_str);
                                        updated = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some("MasterServer") => {
                    if let Some(master_server) = config.get("masterServer") {
                        match key {
                            "enabled" => {
                                if let Some(value) = master_server.get("enabled") {
                                    if let Some(enabled) = value.as_bool() {
                                        *line = format!("enabled = {}", enabled);
                                        updated = true;
                                    }
                                }
                            }
                            "address" => {
                                if let Some(value) = master_server.get("address") {
                                    if let Some(address) = value.as_str() {
                                        *line = format!("address = {}", address);
                                        updated = true;
                                    }
                                }
                            }
                            "port" => {
                                if let Some(value) = master_server.get("port") {
                                    if let Some(port) = value.as_u64() {
                                        *line = format!("port = {}", port);
                                        updated = true;
                                    }
                                }
                            }
                            "rate" => {
                                if let Some(value) = master_server.get("rate") {
                                    if let Some(rate) = value.as_u64() {
                                        *line = format!("rate = {}", rate);
                                        updated = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if !updated {
        return Err("No matching configuration keys found to update".to_string());
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
async fn get_tes3mp_server_config() -> Result<serde_json::Value, String> {
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
    let parsed_config = parse_server_config(&config_content)?;

    // Convert to JSON
    let json_config = serde_json::to_value(parsed_config)
        .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

    Ok(json_config)
}

#[tauri::command]
async fn set_tes3mp_server_config(config: serde_json::Value) -> Result<bool, String> {
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
    let updated_content = update_server_config_values(&config_content, &config)?;

    // Write the updated config back to the file
    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write updated TES3MP server config: {}", e))?;

    log::info!("Successfully updated TES3MP server config");
    Ok(true)
}

#[tauri::command]
async fn get_tes3mp_server_settings() -> Result<serde_json::Value, String> {
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
    let parsed_config = parse_server_settings(&config_content)?;

    // Convert to JSON
    let json_config = serde_json::to_value(parsed_config)
        .map_err(|e| format!("Failed to serialize settings to JSON: {}", e))?;

    Ok(json_config)
}

#[tauri::command]
async fn set_tes3mp_server_settings(settings: serde_json::Value) -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Construct the path to the TES3MP server config file
    let config_path = appdata_dir
        .join("TES3MP")
        .join("server")
        .join("scripts")
        .join("config.lua");

    // Parse the JSON settings into our struct
    let server_settings: ServerSettings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

    // Generate the Lua config content
    let config_content = write_server_settings(&server_settings)?;

    // Write the config file
    fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write TES3MP server settings: {}", e))?;

    log::info!("TES3MP server settings saved successfully");
    Ok(true)
}
