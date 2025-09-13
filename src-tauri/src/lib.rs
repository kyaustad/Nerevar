use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use zip::ZipArchive;
use serde::{Deserialize, Serialize};
use dirs;
use tauri::Emitter;

#[derive(Serialize, Deserialize)]
struct NerevarConfig {
    tes3mp_path: String,
    version: String,
    last_updated: String,
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
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .invoke_handler(tauri::generate_handler![download_latest_windows_release, get_nerevar_config, get_openmw_config, run_openmw_wizard, run_openmw_launcher])
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
async fn download_latest_windows_release(url: String) -> Result<String, String> {
    // Use temp directory to avoid Tauri rebuilds
    let temp_dir = std::env::temp_dir();
    
    let zip_filename = "tes3mp_latest.zip";
    let extract_dir = "tes3mp_extracted";
    
    let zip_path = temp_dir.join(zip_filename);
    let extract_path = temp_dir.join(extract_dir);

    log::info!("Using temp directory: {}", temp_dir.display());
    log::info!("Zip will be saved to: {}", zip_path.display());
    log::info!("Will extract to: {}", extract_path.display());

    // Step 1: Download the zip file
    log::info!("Downloading TES3MP from: {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    // Step 2: Save the zip file
    fs::write(&zip_path, bytes)
        .map_err(|e| format!("Failed to save zip file: {}", e))?;

    log::info!("Zip file saved as: {}", zip_path.display());

    // Step 3: Extract the zip file
    let zip_file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive = ZipArchive::new(BufReader::new(zip_file))
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    // Create extraction directory
    if extract_path.exists() {
        log::info!("Removing existing extraction directory: {}", extract_path.display());
        fs::remove_dir_all(&extract_path)
            .map_err(|e| format!("Failed to remove existing extraction directory: {}", e))?;
    }

    log::info!("Extracting zip file to: {}", extract_path.display());
    archive
        .extract(&extract_path)
        .map_err(|e| format!("Failed to extract zip file: {}", e))?;

    log::info!("Zip file extracted to: {}", extract_path.display());

    // Step 4: Delete the zip file
    fs::remove_file(&zip_path)
        .map_err(|e| format!("Failed to delete zip file: {}", e))?;

    log::info!("Zip file deleted: {}", zip_path.display());

    // Step 5: Check if tes3mp.exe is directly in the extracted folder
    let exe_path = extract_path.join("tes3mp.exe");
    
    log::info!("Checking for tes3mp.exe directly in: {}", extract_path.display());
    
    let (tes3mp_folder, final_exe_path) = if exe_path.exists() {
        log::info!("Found tes3mp.exe directly in extraction folder: {}", exe_path.display());
        log::info!("Using extraction folder as TES3MP folder: {}", extract_path.display());
        (extract_path, exe_path)
    } else {
        // Step 6: Look for TES3MP folder (should be named something like tes3mp.Win64.release.0.8.1)
        log::info!("tes3mp.exe not found directly, looking for TES3MP folder in: {}", extract_path.display());
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
        return Err(format!("tes3mp.exe not found in: {}", tes3mp_folder.display()));
    }

    log::info!("Found tes3mp.exe at: {}", final_exe_path.display());
    log::info!("TES3MP installation verified in: {}", tes3mp_folder.display());

    // Step 7: Move TES3MP to permanent AppData location
    let appdata_dir = get_appdata_dir()?;
    let permanent_tes3mp_path = appdata_dir.join("TES3MP");
    
    log::info!("Moving TES3MP to permanent location: {}", permanent_tes3mp_path.display());
    
    // Remove existing TES3MP folder if it exists
    if permanent_tes3mp_path.exists() {
        log::info!("Removing existing TES3MP installation: {}", permanent_tes3mp_path.display());
        fs::remove_dir_all(&permanent_tes3mp_path)
            .map_err(|e| format!("Failed to remove existing TES3MP installation: {}", e))?;
    }
    
    // Move the extracted folder to AppData
    fs::rename(&tes3mp_folder, &permanent_tes3mp_path)
        .map_err(|e| format!("Failed to move TES3MP to AppData: {}", e))?;
    
    log::info!("TES3MP moved to: {}", permanent_tes3mp_path.display());
    
    // Step 8: Create config.json
    let config_path = appdata_dir.join("config.json");
    let config = NerevarConfig {
        tes3mp_path: permanent_tes3mp_path.to_string_lossy().to_string(),
        version: "0.8.1".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
    };
    
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    log::info!("Config saved to: {}", config_path.display());

    Ok(format!("TES3MP installed successfully to: {}", permanent_tes3mp_path.display()))
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

    log::error!("No TES3MP folder found. Available folders: {:?}", found_folders);
    Err(format!("No TES3MP folder found in: {}. Available folders: {:?}", extract_path.display(), found_folders))
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
    
    log::info!("Loaded TES3MP config: {}", config.tes3mp_path);
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
                serde_json::Value::Number(serde_json::Number::from(clean_value.parse::<i64>().unwrap()))
            } else if clean_value.parse::<f64>().is_ok() {
                serde_json::Value::Number(serde_json::Number::from_f64(clean_value.parse::<f64>().unwrap()).unwrap())
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
    let nerevar_config = get_nerevar_config()
        .map_err(|e| format!("Failed to get Nerevar config: {}", e))?;
    
    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;
    
    // Construct the path to the OpenMW wizard executable
    let openmw_wizard_path = Path::new(&tes3mp_path).join("openmw-wizard.exe");
    
    // Check if the wizard executable exists
    if !openmw_wizard_path.exists() {
        return Err(format!("OpenMW wizard not found at: {}", openmw_wizard_path.display()));
    }
    
    log::info!("Running OpenMW wizard at: {}", openmw_wizard_path.display());
    
    // Spawn the OpenMW wizard process
    let mut child = std::process::Command::new(&openmw_wizard_path)
        .spawn()
        .map_err(|e| format!("Failed to run OpenMW wizard: {}", e))?;
    
    let pid = child.id();
    log::info!("OpenMW wizard started successfully (PID: {})", pid);
    
    // Send initial event that wizard started
    app_handle.emit("openmw-wizard-started", &pid)
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
                
                log::info!("OpenMW wizard (PID: {}) exited with status: {:?}", pid, status.code());
                
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
    let nerevar_config = get_nerevar_config()
        .map_err(|e| format!("Failed to get Nerevar config: {}", e))?;
    
    let config = nerevar_config.ok_or("No Nerevar config found. Please install TES3MP first.")?;
    let tes3mp_path = config.tes3mp_path;
    
    // Construct the path to the OpenMW wizard executable
    let openmw_launcher_path = Path::new(&tes3mp_path).join("openmw-launcher.exe");
    
    // Check if the wizard executable exists
    if !openmw_launcher_path.exists() {
        return Err(format!("OpenMW launcher not found at: {}", openmw_launcher_path.display()));
    }
    
    log::info!("Running OpenMW launcher at: {}", openmw_launcher_path.display());
    
    // Spawn the OpenMW wizard process
    let mut child = std::process::Command::new(&openmw_launcher_path)
        .spawn()
        .map_err(|e| format!("Failed to run OpenMW launcher: {}", e))?;
    
    let pid = child.id();
    log::info!("OpenMW launcher started successfully (PID: {})", pid);
    
    // Send initial event that wizard started
    app_handle.emit("openmw-launcher-started", &pid)
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
                
                log::info!("OpenMW launcher (PID: {}) exited with status: {:?}", pid, status.code());
                
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
    
    Ok(format!("OpenMW launcher started successfully (PID: {})", pid))
}