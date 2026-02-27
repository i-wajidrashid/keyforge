//! Tauri command handlers.
//!
//! Thin wrappers around the `keyforge-crypto` and `keyforge-vault` crates
//! exposed to the frontend via `tauri::command`.  Every command returns
//! `Result<T, String>` following Tauri's error convention.

/// Create a brand-new encrypted vault.
#[tauri::command]
pub fn vault_create(_password: String) -> Result<String, String> {
    // Phase 1 stub — real implementation will derive keys and create SQLCipher DB.
    Ok("vault_created".into())
}

/// Unlock the vault with the master password.
#[tauri::command]
pub fn vault_unlock(_password: String) -> Result<bool, String> {
    Ok(true)
}

/// Lock the vault (zeroize key from memory).
#[tauri::command]
pub fn vault_lock() -> Result<(), String> {
    Ok(())
}

/// Check whether the vault is currently locked.
#[tauri::command]
pub fn vault_is_locked() -> Result<bool, String> {
    Ok(true)
}

/// Generate a TOTP code via the Rust crypto crate.
#[tauri::command]
pub fn otp_generate_totp(
    secret: String,
    algorithm: String,
    digits: u32,
    period: u64,
) -> Result<String, String> {
    let secret_bytes = base32_decode(&secret).ok_or_else(|| "Invalid Base32 secret".to_string())?;

    let algo = match algorithm.as_str() {
        "SHA1" => keyforge_crypto::hotp::Algorithm::SHA1,
        "SHA256" => keyforge_crypto::hotp::Algorithm::SHA256,
        "SHA512" => keyforge_crypto::hotp::Algorithm::SHA512,
        other => return Err(format!("Unsupported algorithm: {other}")),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let code = keyforge_crypto::totp::generate(&secret_bytes, now, period, digits, algo);
    Ok(code)
}

/// Return basic platform information.
#[tauri::command]
pub fn platform_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    }))
}

// ── helpers ──────────────────────────────────────────────────────────

fn base32_decode(input: &str) -> Option<Vec<u8>> {
    base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &input.to_uppercase().replace([' ', '-'], ""),
    )
}
