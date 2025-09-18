use dirs;
use std::fs;
use std::path::Path;

pub fn get_documents_folder() -> Result<std::path::PathBuf, String> {
    if let Some(documents_dir) = dirs::document_dir() {
        log::info!("Documents directory: {}", documents_dir.display());
        return Ok(documents_dir);
    } else {
        Err(format!("Failed to get documents directory"))
    }
}

pub fn find_tes3mp_folder(extract_path: &Path) -> Result<std::path::PathBuf, String> {
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

pub fn get_appdata_dir() -> Result<std::path::PathBuf, String> {
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
