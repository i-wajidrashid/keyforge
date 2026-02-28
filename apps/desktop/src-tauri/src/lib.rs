mod commands;

use commands::{
    otp_generate_hotp, otp_generate_totp, otp_generate_totp_raw, platform_info, token_add,
    token_delete, token_increment_counter, token_list, token_reorder, token_update, vault_create,
    vault_exists, vault_export_encrypted, vault_export_uris, vault_import_encrypted,
    vault_import_uris, vault_is_locked, vault_lock, vault_unlock, AppState,
};

/// Build and configure the Tauri application.
///
/// This is the single entry-point consumed by both the binary (`main.rs`)
/// and Tauri's mobile init path.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_biometry::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Vault lifecycle
            vault_create,
            vault_unlock,
            vault_lock,
            vault_is_locked,
            vault_exists,
            // Token CRUD
            token_list,
            token_add,
            token_delete,
            token_update,
            token_reorder,
            token_increment_counter,
            // OTP generation
            otp_generate_totp,
            otp_generate_totp_raw,
            otp_generate_hotp,
            // Import / Export
            vault_import_uris,
            vault_export_uris,
            vault_export_encrypted,
            vault_import_encrypted,
            // Platform
            platform_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeyForge");
}
