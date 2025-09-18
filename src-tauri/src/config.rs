use crate::types::{Mode, NerevarConfig, OpenMWConfig};
use crate::utils::{get_appdata_dir, get_documents_folder};
use std::fs;

pub fn get_nerevar_config() -> Result<Option<NerevarConfig>, String> {
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

pub fn get_openmw_config() -> Result<Option<OpenMWConfig>, String> {
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

pub fn parse_openmw_config(content: &str) -> Result<OpenMWConfig, String> {
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

pub fn set_mode(mode: Mode) -> Result<String, String> {
    let appdata_dir = get_appdata_dir()?;
    let config_path = appdata_dir.join("config.json");

    // Try to read existing config
    let mut config = if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_json::from_str::<NerevarConfig>(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?
    } else {
        return Err("No config file found. Please install TES3MP first.".to_string());
    };

    // Update the mode
    config.mode = Some(mode);

    // Write the updated config back
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Updated mode to: {:?}", mode);
    Ok(format!("Mode set to: {:?}", mode))
}

pub fn update_config_values(
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

pub fn update_server_config_values(
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
