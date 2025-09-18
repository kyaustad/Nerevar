// Import our modules
mod commands;
mod config;
mod parsers;
mod types;
mod utils;

// Re-export types for external use
pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api_url = "https://nerevar.cc/api/v1";
    log::info!("Nerevar API URL: {}", api_url);
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .invoke_handler(tauri::generate_handler![
            commands::download_latest_windows_release,
            commands::get_nerevar_config,
            commands::get_openmw_config,
            commands::run_openmw_wizard,
            commands::run_openmw_launcher,
            commands::check_for_tes3mp_update,
            commands::set_mode,
            commands::get_app_version,
            commands::check_for_app_update,
            commands::download_app_update,
            commands::apply_app_update,
            commands::run_tes3mp_browser,
            commands::run_tes3mp,
            commands::ping_server_tcp,
            commands::set_tes3mp_client_config,
            commands::get_tes3mp_server_config,
            commands::set_tes3mp_server_config,
            commands::get_tes3mp_server_settings,
            commands::set_tes3mp_server_settings,
            commands::run_tes3mp_server,
            commands::open_config_lua_in_explorer,
            commands::open_nerevar_appdata_dir_in_explorer,
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
