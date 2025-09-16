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
use open;

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
            run_tes3mp_server,
            open_config_lua_in_explorer,
            open_nerevar_appdata_dir_in_explorer,
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
    #[serde(rename = "gameMode")]
    game_mode: String,
    #[serde(rename = "loginTime")]
    login_time: i32,
    #[serde(rename = "maxClientsPerIP")]
    max_clients_per_ip: i32,
    difficulty: i32,
    #[serde(rename = "gameSettings")]
    game_settings: Vec<GameSetting>,

    #[serde(rename = "defaultTimeTable")]
    default_time_table: DefaultTimeTable,

    #[serde(rename = "passTimeWhenEmpty")]
    pass_time_when_empty: bool,
    #[serde(rename = "nightStartHour")]
    night_start_hour: i32,
    #[serde(rename = "nightEndHour")]
    night_end_hour: i32,
    #[serde(rename = "allowConsole")]
    allow_console: bool,
    #[serde(rename = "allowBedRest")]
    allow_bed_rest: bool,
    #[serde(rename = "allowWildernessRest")]
    allow_wilderness_rest: bool,
    #[serde(rename = "allowWait")]
    allow_wait: bool,
    #[serde(rename = "shareJournal")]
    share_journal: bool,
    #[serde(rename = "shareFactionRanks")]
    share_faction_ranks: bool,
    #[serde(rename = "shareFactionExpulsion")]
    share_faction_expulsion: bool,
    #[serde(rename = "shareFactionReputation")]
    share_faction_reputation: bool,
    #[serde(rename = "shareTopics")]
    share_topics: bool,
    #[serde(rename = "shareBounty")]
    share_bounty: bool,
    #[serde(rename = "shareReputation")]
    share_reputation: bool,
    #[serde(rename = "shareMapExploration")]
    share_map_exploration: bool,
    #[serde(rename = "shareVideos")]
    share_videos: bool,
    #[serde(rename = "useInstancedSpawn")]
    use_instanced_spawn: bool,

    #[serde(rename = "respawnAtImperialShrine")]
    respawn_at_imperial_shrine: bool,
    #[serde(rename = "respawnAtTribunalTemple")]
    respawn_at_tribunal_temple: bool,

    #[serde(rename = "maxAttributeValue")]
    max_attribute_value: i32,
    #[serde(rename = "maxSpeedValue")]
    max_speed_value: i32,
    #[serde(rename = "maxSkillValue")]
    max_skill_value: i32,
    #[serde(rename = "maxAcrobaticsValue")]
    max_acrobatics_value: i32,
    #[serde(rename = "ignoreModifierWithMaxSkill")]
    ignore_modifier_with_max_skill: bool,

    #[serde(rename = "playersRespawn")]
    players_respawn: bool,
    #[serde(rename = "deathTime")]
    death_time: i32,
    #[serde(rename = "deathPenaltyJailDays")]
    death_penalty_jail_days: i32,
    #[serde(rename = "bountyResetOnDeath")]
    bounty_reset_on_death: bool,
    #[serde(rename = "bountyDeathPenalty")]
    bounty_death_penalty: bool,
    #[serde(rename = "allowSuicideCommand")]
    allow_suicide_command: bool,
    #[serde(rename = "allowFixmeCommand")]
    allow_fixme_command: bool,
    #[serde(rename = "fixmeInterval")]
    fixme_interval: i32,
    #[serde(rename = "rankColors")]
    rank_colors: RankColors,
    #[serde(rename = "pingDifferenceRequiredForAuthority")]
    ping_difference_required_for_authority: i32,
    #[serde(rename = "enforcedLogLevel")]
    enforced_log_level: i32,
    #[serde(rename = "physicsFramerate")]
    physics_framerate: i32,
    #[serde(rename = "allowOnContainerForUnloadedCells")]
    allow_on_container_for_unloaded_cells: bool,
    #[serde(rename = "enablePlayerCollision")]
    enable_player_collision: bool,
    #[serde(rename = "enableActorCollision")]
    enable_actor_collision: bool,
    #[serde(rename = "enablePlacedObjectCollision")]
    enable_placed_object_collision: bool,

    #[serde(rename = "useActorCollisionForPlacedObjects")]
    use_actor_collision_for_placed_objects: bool,
    #[serde(rename = "maximumObjectScale")]
    maximum_object_scale: f64,
    #[serde(rename = "enforceDataFiles")]
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
    #[serde(rename = "daysPassed")]
    days_passed: i32,
    #[serde(rename = "dayTimeScale")]
    day_time_scale: i32,
    #[serde(rename = "nightTimeScale")]
    night_time_scale: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpawnLocation {
    #[serde(rename = "cellDescription")]
    cell_description: String,
    position: Vec<f64>,
    rotation: Vec<f64>,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RespawnLocation {
    #[serde(rename = "cellDescription")]
    cell_description: String,
    position: Vec<f64>,
    rotation: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RankColors {
    #[serde(rename = "serverOwner")]
    server_owner: String,
    admin: String,
    moderator: String,
}

fn parse_server_settings(content: &str) -> Result<ServerSettings, String> {
    // BRUTE FORCE: Search for the specific fields we need
    let max_attribute_value = if let Some(start) = content.find("config.maxAttributeValue = ") {
        let line_start = start + "config.maxAttributeValue = ".len();
        let line_end = content[line_start..]
            .find('\n')
            .unwrap_or(content.len() - line_start);
        let value_str = content[line_start..line_start + line_end].trim();
        value_str.parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    let players_respawn = if let Some(start) = content.find("config.playersRespawn = ") {
        let line_start = start + "config.playersRespawn = ".len();
        let line_end = content[line_start..]
            .find('\n')
            .unwrap_or(content.len() - line_start);
        let value_str = content[line_start..line_start + line_end].trim();
        value_str == "true"
    } else {
        false
    };

    let mut config = ConfigSettings {
        game_mode: String::new(),
        login_time: 0,
        max_clients_per_ip: 0,
        difficulty: 0,
        game_settings: Vec::new(),
        default_time_table: DefaultTimeTable {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            days_passed: 0,
            day_time_scale: 0,
            night_time_scale: 0,
        },
        pass_time_when_empty: false,
        night_start_hour: 0,
        night_end_hour: 0,
        allow_console: false,
        allow_bed_rest: false,
        allow_wilderness_rest: false,
        allow_wait: false,
        share_journal: false,
        share_faction_ranks: false,
        share_faction_expulsion: false,
        share_faction_reputation: false,
        share_topics: false,
        share_bounty: false,
        share_reputation: false,
        share_map_exploration: false,
        share_videos: false,
        use_instanced_spawn: false,
        respawn_at_imperial_shrine: false,
        respawn_at_tribunal_temple: false,
        max_attribute_value,
        max_speed_value: 0,
        max_skill_value: 0,
        max_acrobatics_value: 0,
        ignore_modifier_with_max_skill: false,
        players_respawn,
        death_time: 0,
        death_penalty_jail_days: 0,
        bounty_reset_on_death: false,
        bounty_death_penalty: false,
        allow_suicide_command: false,
        allow_fixme_command: false,
        fixme_interval: 0,
        rank_colors: RankColors {
            server_owner: String::new(),
            admin: String::new(),
            moderator: String::new(),
        },
        ping_difference_required_for_authority: 0,
        enforced_log_level: 0,
        physics_framerate: 0,
        allow_on_container_for_unloaded_cells: false,
        enable_player_collision: false,
        enable_actor_collision: false,
        enable_placed_object_collision: false,
        use_actor_collision_for_placed_objects: false,
        maximum_object_scale: 0.0,
        enforce_data_files: false,
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("--") {
            i += 1;
            continue;
        }

        // Parse simple key-value pairs
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            match key {
                "config.gameMode" => {
                    config.game_mode = parse_string_value(value)?;
                }
                "config.loginTime" => {
                    config.login_time = parse_number_value(value)?;
                }
                "config.maxClientsPerIP" => {
                    config.max_clients_per_ip = parse_number_value(value)?;
                }
                "config.difficulty" => {
                    config.difficulty = parse_number_value(value)?;
                }
                "config.passTimeWhenEmpty" => {
                    config.pass_time_when_empty = parse_bool_value(value)?;
                }
                "config.nightStartHour" => {
                    config.night_start_hour = parse_number_value(value)?;
                }
                "config.nightEndHour" => {
                    config.night_end_hour = parse_number_value(value)?;
                }
                "config.allowConsole" => {
                    config.allow_console = parse_bool_value(value)?;
                }
                "config.allowBedRest" => {
                    config.allow_bed_rest = parse_bool_value(value)?;
                }
                "config.allowWildernessRest" => {
                    config.allow_wilderness_rest = parse_bool_value(value)?;
                }
                "config.allowWait" => {
                    config.allow_wait = parse_bool_value(value)?;
                }
                "config.shareJournal" => {
                    config.share_journal = parse_bool_value(value)?;
                }
                "config.shareFactionRanks" => {
                    config.share_faction_ranks = parse_bool_value(value)?;
                }
                "config.shareFactionExpulsion" => {
                    config.share_faction_expulsion = parse_bool_value(value)?;
                }
                "config.shareFactionReputation" => {
                    config.share_faction_reputation = parse_bool_value(value)?;
                }
                "config.shareTopics" => {
                    config.share_topics = parse_bool_value(value)?;
                }
                "config.shareBounty" => {
                    config.share_bounty = parse_bool_value(value)?;
                }
                "config.shareReputation" => {
                    config.share_reputation = parse_bool_value(value)?;
                }
                "config.shareMapExploration" => {
                    config.share_map_exploration = parse_bool_value(value)?;
                }
                "config.shareVideos" => {
                    config.share_videos = parse_bool_value(value)?;
                }
                "config.useInstancedSpawn" => {
                    config.use_instanced_spawn = parse_bool_value(value)?;
                }
                "config.respawnAtImperialShrine" => {
                    config.respawn_at_imperial_shrine = parse_bool_value(value)?;
                }
                "config.respawnAtTribunalTemple" => {
                    config.respawn_at_tribunal_temple = parse_bool_value(value)?;
                }
                "config.maxAttributeValue" => {
                    config.max_attribute_value = parse_number_value(value)?;
                }
                "config.maxSpeedValue" => {
                    config.max_speed_value = parse_number_value(value)?;
                }
                "config.maxSkillValue" => {
                    config.max_skill_value = parse_number_value(value)?;
                }
                "config.maxAcrobaticsValue" => {
                    config.max_acrobatics_value = parse_number_value(value)?;
                }
                "config.ignoreModifierWithMaxSkill" => {
                    config.ignore_modifier_with_max_skill = parse_bool_value(value)?;
                }
                "config.playersRespawn" => {
                    config.players_respawn = parse_bool_value(value)?;
                }
                "config.deathTime" => {
                    config.death_time = parse_number_value(value)?;
                }
                "config.deathPenaltyJailDays" => {
                    config.death_penalty_jail_days = parse_number_value(value)?;
                }
                "config.bountyResetOnDeath" => {
                    config.bounty_reset_on_death = parse_bool_value(value)?;
                }
                "config.bountyDeathPenalty" => {
                    config.bounty_death_penalty = parse_bool_value(value)?;
                }
                "config.allowSuicideCommand" => {
                    config.allow_suicide_command = parse_bool_value(value)?;
                }
                "config.allowFixmeCommand" => {
                    config.allow_fixme_command = parse_bool_value(value)?;
                }
                "config.fixmeInterval" => {
                    config.fixme_interval = parse_number_value(value)?;
                }
                "config.pingDifferenceRequiredForAuthority" => {
                    config.ping_difference_required_for_authority = parse_number_value(value)?;
                }
                "config.enforcedLogLevel" => {
                    config.enforced_log_level = parse_number_value(value)?;
                }
                "config.physicsFramerate" => {
                    config.physics_framerate = parse_number_value(value)?;
                }
                "config.allowOnContainerForUnloadedCells" => {
                    config.allow_on_container_for_unloaded_cells = parse_bool_value(value)?;
                }
                "config.enablePlayerCollision" => {
                    config.enable_player_collision = parse_bool_value(value)?;
                }
                "config.enableActorCollision" => {
                    config.enable_actor_collision = parse_bool_value(value)?;
                }
                "config.enablePlacedObjectCollision" => {
                    config.enable_placed_object_collision = parse_bool_value(value)?;
                }
                "config.useActorCollisionForPlacedObjects" => {
                    config.use_actor_collision_for_placed_objects = parse_bool_value(value)?;
                }
                "config.maximumObjectScale" => {
                    config.maximum_object_scale = parse_float_value(value)?;
                }
                "config.enforceDataFiles" => {
                    config.enforce_data_files = parse_bool_value(value)?;
                }
                "config.gameSettings" => {
                    // Parse game settings table
                    if let Some(table_content) = parse_table_content(&lines, &mut i)? {
                        config.game_settings = parse_game_settings_table(&table_content)?;
                    }
                }

                "config.defaultTimeTable" => {
                    // Parse default time table
                    if let Some(table_content) = parse_table_content(&lines, &mut i)? {
                        config.default_time_table = parse_time_table(&table_content)?;
                    }
                }

                _ => {
                    // Unknown key, skip
                }
            }
        }

        i += 1;
    }

    Ok(ServerSettings { config })
}

// Helper functions for parsing Lua values
fn parse_string_value(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        Ok(trimmed[1..trimmed.len() - 1].to_string())
    } else if trimmed.starts_with('"') && trimmed.contains("..") {
        // Handle concatenated strings like "text" .. "more text"
        let mut result = String::new();
        let parts: Vec<&str> = trimmed.split("..").collect();

        for part in parts {
            let part = part.trim();
            if part.starts_with('"') && part.ends_with('"') {
                result.push_str(&part[1..part.len() - 1]);
            } else if part.starts_with('"') {
                result.push_str(&part[1..]);
            } else if part.ends_with('"') {
                result.push_str(&part[..part.len() - 1]);
            }
        }
        Ok(result)
    } else {
        Err(format!("Invalid string value: {}", value))
    }
}

fn parse_number_value(value: &str) -> Result<i32, String> {
    let trimmed = value.trim();
    trimmed
        .parse::<i32>()
        .map_err(|e| format!("Invalid number value '{}': {}", value, e))
}

fn parse_float_value(value: &str) -> Result<f64, String> {
    let trimmed = value.trim();
    trimmed
        .parse::<f64>()
        .map_err(|e| format!("Invalid float value '{}': {}", value, e))
}

fn parse_bool_value(value: &str) -> Result<bool, String> {
    let trimmed = value.trim();
    match trimmed {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("Invalid boolean value: {}", value)),
    }
}

fn parse_table_content(lines: &[&str], index: &mut usize) -> Result<Option<String>, String> {
    let mut content = String::new();
    let mut brace_count = 0;
    let mut found_opening = false;

    while *index < lines.len() {
        let line = lines[*index].trim();

        if line.is_empty() || line.starts_with("--") {
            *index += 1;
            continue;
        }

        if !found_opening {
            if line.contains('{') {
                found_opening = true;
                let brace_start = line.find('{').unwrap();
                content.push_str(&line[brace_start..]);
                brace_count += line[brace_start..].matches('{').count() as i32;
                brace_count -= line[brace_start..].matches('}').count() as i32;
            }
        } else {
            content.push_str(line);
            content.push('\n');
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;

            if brace_count <= 0 {
                break;
            }
        }

        *index += 1;
    }

    if found_opening {
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

fn parse_game_settings_table(content: &str) -> Result<Vec<GameSetting>, String> {
    let mut settings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed == "{" || trimmed == "}" {
            continue;
        }

        // Parse lines like: { name = "best attack", value = false },
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let mut name = String::new();
            let mut value = serde_json::Value::Bool(false);

            for part in inner.split(',') {
                let part = part.trim();
                if part.starts_with("name =") {
                    let name_part = &part[6..].trim();
                    name = parse_string_value(name_part)?;
                } else if part.starts_with("value =") {
                    let value_part = &part[7..].trim();
                    if *value_part == "true" || *value_part == "false" {
                        value = serde_json::Value::Bool(parse_bool_value(value_part)?);
                    } else {
                        value = serde_json::Value::Number(parse_number_value(value_part)?.into());
                    }
                }
            }

            if !name.is_empty() {
                settings.push(GameSetting { name, value });
            }
        }
    }

    Ok(settings)
}

fn parse_time_table(content: &str) -> Result<DefaultTimeTable, String> {
    let mut time_table = DefaultTimeTable {
        year: 0,
        month: 0,
        day: 0,
        hour: 0,
        days_passed: 0,
        day_time_scale: 0,
        night_time_scale: 0,
    };

    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed == "{" || trimmed == "}" {
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim().trim_end_matches(',');

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

fn update_server_settings_values(
    content: &str,
    config: &serde_json::Value,
) -> Result<String, String> {
    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut updated = false;

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        // Check if this line contains a key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();

            // Update values based on the key
            match key {
                "config.gameMode" => {
                    if let Some(value) = config.get("gameMode") {
                        if let Some(game_mode) = value.as_str() {
                            *line = format!("config.gameMode = \"{}\"", game_mode);
                            updated = true;
                        }
                    }
                }
                "config.loginTime" => {
                    if let Some(value) = config.get("loginTime") {
                        if let Some(login_time) = value.as_i64() {
                            *line = format!("config.loginTime = {}", login_time);
                            updated = true;
                        }
                    }
                }
                "config.maxClientsPerIP" => {
                    if let Some(value) = config.get("maxClientsPerIP") {
                        if let Some(max_clients) = value.as_i64() {
                            *line = format!("config.maxClientsPerIP = {}", max_clients);
                            updated = true;
                        }
                    }
                }
                "config.difficulty" => {
                    if let Some(value) = config.get("difficulty") {
                        if let Some(difficulty) = value.as_i64() {
                            *line = format!("config.difficulty = {}", difficulty);
                            updated = true;
                        }
                    }
                }
                "config.passTimeWhenEmpty" => {
                    if let Some(value) = config.get("passTimeWhenEmpty") {
                        if let Some(pass_time) = value.as_bool() {
                            *line = format!("config.passTimeWhenEmpty = {}", pass_time);
                            updated = true;
                        }
                    }
                }
                "config.nightStartHour" => {
                    if let Some(value) = config.get("nightStartHour") {
                        if let Some(hour) = value.as_i64() {
                            *line = format!("config.nightStartHour = {}", hour);
                            updated = true;
                        }
                    }
                }
                "config.nightEndHour" => {
                    if let Some(value) = config.get("nightEndHour") {
                        if let Some(hour) = value.as_i64() {
                            *line = format!("config.nightEndHour = {}", hour);
                            updated = true;
                        }
                    }
                }
                "config.allowConsole" => {
                    if let Some(value) = config.get("allowConsole") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowConsole = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.allowBedRest" => {
                    if let Some(value) = config.get("allowBedRest") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowBedRest = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.allowWildernessRest" => {
                    if let Some(value) = config.get("allowWildernessRest") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowWildernessRest = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.allowWait" => {
                    if let Some(value) = config.get("allowWait") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowWait = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.shareJournal" => {
                    if let Some(value) = config.get("shareJournal") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareJournal = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareFactionRanks" => {
                    if let Some(value) = config.get("shareFactionRanks") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareFactionRanks = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareFactionExpulsion" => {
                    if let Some(value) = config.get("shareFactionExpulsion") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareFactionExpulsion = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareFactionReputation" => {
                    if let Some(value) = config.get("shareFactionReputation") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareFactionReputation = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareTopics" => {
                    if let Some(value) = config.get("shareTopics") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareTopics = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareBounty" => {
                    if let Some(value) = config.get("shareBounty") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareBounty = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareReputation" => {
                    if let Some(value) = config.get("shareReputation") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareReputation = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareMapExploration" => {
                    if let Some(value) = config.get("shareMapExploration") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareMapExploration = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.shareVideos" => {
                    if let Some(value) = config.get("shareVideos") {
                        if let Some(share) = value.as_bool() {
                            *line = format!("config.shareVideos = {}", share);
                            updated = true;
                        }
                    }
                }
                "config.useInstancedSpawn" => {
                    if let Some(value) = config.get("useInstancedSpawn") {
                        if let Some(use_instanced) = value.as_bool() {
                            *line = format!("config.useInstancedSpawn = {}", use_instanced);
                            updated = true;
                        }
                    }
                }
                "config.respawnAtImperialShrine" => {
                    if let Some(value) = config.get("respawnAtImperialShrine") {
                        if let Some(respawn) = value.as_bool() {
                            *line = format!("config.respawnAtImperialShrine = {}", respawn);
                            updated = true;
                        }
                    }
                }
                "config.respawnAtTribunalTemple" => {
                    if let Some(value) = config.get("respawnAtTribunalTemple") {
                        if let Some(respawn) = value.as_bool() {
                            *line = format!("config.respawnAtTribunalTemple = {}", respawn);
                            updated = true;
                        }
                    }
                }
                "config.maxAttributeValue" => {
                    if let Some(value) = config.get("maxAttributeValue") {
                        if let Some(max_val) = value.as_i64() {
                            *line = format!("config.maxAttributeValue = {}", max_val);
                            updated = true;
                        }
                    }
                }
                "config.maxSpeedValue" => {
                    if let Some(value) = config.get("maxSpeedValue") {
                        if let Some(max_val) = value.as_i64() {
                            *line = format!("config.maxSpeedValue = {}", max_val);
                            updated = true;
                        }
                    }
                }
                "config.maxSkillValue" => {
                    if let Some(value) = config.get("maxSkillValue") {
                        if let Some(max_val) = value.as_i64() {
                            *line = format!("config.maxSkillValue = {}", max_val);
                            updated = true;
                        }
                    }
                }
                "config.maxAcrobaticsValue" => {
                    if let Some(value) = config.get("maxAcrobaticsValue") {
                        if let Some(max_val) = value.as_i64() {
                            *line = format!("config.maxAcrobaticsValue = {}", max_val);
                            updated = true;
                        }
                    }
                }
                "config.ignoreModifierWithMaxSkill" => {
                    if let Some(value) = config.get("ignoreModifierWithMaxSkill") {
                        if let Some(ignore) = value.as_bool() {
                            *line = format!("config.ignoreModifierWithMaxSkill = {}", ignore);
                            updated = true;
                        }
                    }
                }
                "config.playersRespawn" => {
                    if let Some(value) = config.get("playersRespawn") {
                        if let Some(respawn) = value.as_bool() {
                            *line = format!("config.playersRespawn = {}", respawn);
                            updated = true;
                        }
                    }
                }
                "config.deathTime" => {
                    if let Some(value) = config.get("deathTime") {
                        if let Some(time) = value.as_i64() {
                            *line = format!("config.deathTime = {}", time);
                            updated = true;
                        }
                    }
                }
                "config.deathPenaltyJailDays" => {
                    if let Some(value) = config.get("deathPenaltyJailDays") {
                        if let Some(days) = value.as_i64() {
                            *line = format!("config.deathPenaltyJailDays = {}", days);
                            updated = true;
                        }
                    }
                }
                "config.bountyResetOnDeath" => {
                    if let Some(value) = config.get("bountyResetOnDeath") {
                        if let Some(reset) = value.as_bool() {
                            *line = format!("config.bountyResetOnDeath = {}", reset);
                            updated = true;
                        }
                    }
                }
                "config.bountyDeathPenalty" => {
                    if let Some(value) = config.get("bountyDeathPenalty") {
                        if let Some(penalty) = value.as_bool() {
                            *line = format!("config.bountyDeathPenalty = {}", penalty);
                            updated = true;
                        }
                    }
                }
                "config.allowSuicideCommand" => {
                    if let Some(value) = config.get("allowSuicideCommand") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowSuicideCommand = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.allowFixmeCommand" => {
                    if let Some(value) = config.get("allowFixmeCommand") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowFixmeCommand = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.fixmeInterval" => {
                    if let Some(value) = config.get("fixmeInterval") {
                        if let Some(interval) = value.as_i64() {
                            *line = format!("config.fixmeInterval = {}", interval);
                            updated = true;
                        }
                    }
                }
                "config.pingDifferenceRequiredForAuthority" => {
                    if let Some(value) = config.get("pingDifferenceRequiredForAuthority") {
                        if let Some(diff) = value.as_i64() {
                            *line = format!("config.pingDifferenceRequiredForAuthority = {}", diff);
                            updated = true;
                        }
                    }
                }
                "config.enforcedLogLevel" => {
                    if let Some(value) = config.get("enforcedLogLevel") {
                        if let Some(level) = value.as_i64() {
                            *line = format!("config.enforcedLogLevel = {}", level);
                            updated = true;
                        }
                    }
                }
                "config.physicsFramerate" => {
                    if let Some(value) = config.get("physicsFramerate") {
                        if let Some(framerate) = value.as_i64() {
                            *line = format!("config.physicsFramerate = {}", framerate);
                            updated = true;
                        }
                    }
                }
                "config.allowOnContainerForUnloadedCells" => {
                    if let Some(value) = config.get("allowOnContainerForUnloadedCells") {
                        if let Some(allow) = value.as_bool() {
                            *line = format!("config.allowOnContainerForUnloadedCells = {}", allow);
                            updated = true;
                        }
                    }
                }
                "config.enablePlayerCollision" => {
                    if let Some(value) = config.get("enablePlayerCollision") {
                        if let Some(enable) = value.as_bool() {
                            *line = format!("config.enablePlayerCollision = {}", enable);
                            updated = true;
                        }
                    }
                }
                "config.enableActorCollision" => {
                    if let Some(value) = config.get("enableActorCollision") {
                        if let Some(enable) = value.as_bool() {
                            *line = format!("config.enableActorCollision = {}", enable);
                            updated = true;
                        }
                    }
                }
                "config.enablePlacedObjectCollision" => {
                    if let Some(value) = config.get("enablePlacedObjectCollision") {
                        if let Some(enable) = value.as_bool() {
                            *line = format!("config.enablePlacedObjectCollision = {}", enable);
                            updated = true;
                        }
                    }
                }
                "config.useActorCollisionForPlacedObjects" => {
                    if let Some(value) = config.get("useActorCollisionForPlacedObjects") {
                        if let Some(use_actor) = value.as_bool() {
                            *line =
                                format!("config.useActorCollisionForPlacedObjects = {}", use_actor);
                            updated = true;
                        }
                    }
                }
                "config.maximumObjectScale" => {
                    if let Some(value) = config.get("maximumObjectScale") {
                        if let Some(scale) = value.as_f64() {
                            *line = format!("config.maximumObjectScale = {}", scale);
                            updated = true;
                        }
                    }
                }
                "config.enforceDataFiles" => {
                    if let Some(value) = config.get("enforceDataFiles") {
                        if let Some(enforce) = value.as_bool() {
                            *line = format!("config.enforceDataFiles = {}", enforce);
                            updated = true;
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
async fn set_tes3mp_server_settings(config: serde_json::Value) -> Result<bool, String> {
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

    // Read the existing config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read TES3MP server settings: {}", e))?;

    // Parse and update the config surgically
    let updated_content = update_server_settings_values(&config_content, &config)?;

    // Write the updated config back to the file
    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write updated TES3MP server settings: {}", e))?;

    log::info!("TES3MP server settings saved successfully");
    Ok(true)
}

#[tauri::command]
async fn run_tes3mp_server(app_handle: tauri::AppHandle) -> Result<String, String> {
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
async fn open_config_lua_in_explorer() -> Result<bool, String> {
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

    // Open the config file in the default file explorer
    open::that(config_path).map_err(|e| format!("Failed to open config file: {}", e))?;

    Ok(true)
}

#[tauri::command]
async fn open_nerevar_appdata_dir_in_explorer() -> Result<bool, String> {
    // Get the AppData directory for Nerevar
    let appdata_dir = get_appdata_dir()?;

    // Open the AppData directory in the default file explorer
    open::that(appdata_dir).map_err(|e| format!("Failed to open AppData directory: {}", e))?;

    Ok(true)
}
