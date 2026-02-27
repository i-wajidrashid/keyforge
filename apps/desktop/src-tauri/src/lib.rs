mod commands;

use commands::{
    otp_generate_totp, platform_info, vault_create, vault_is_locked, vault_lock, vault_unlock,
};

/// Build and configure the Tauri application.
///
/// This is the single entry-point consumed by both the binary (`main.rs`)
/// and Tauri's mobile init path.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            vault_create,
            vault_unlock,
            vault_lock,
            vault_is_locked,
            otp_generate_totp,
            platform_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeyForge");
}
