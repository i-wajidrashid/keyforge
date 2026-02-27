//! Tauri command handlers.
//!
//! Real wrappers around the `keyforge-crypto` and `keyforge-vault` crates
//! exposed to the frontend via `tauri::command`.  Every command returns
//! `Result<T, String>` following Tauri's error convention.
//!
//! The vault is stored in Tauri managed state behind a `Mutex`. When
//! locked, the inner `Option` is `None` (the key material has been
//! zeroized). When unlocked it holds a live `Vault` handle.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use keyforge_crypto::kdf::KdfParams;
use keyforge_crypto::random::generate_salt;
use keyforge_vault::db::Vault;
use keyforge_vault::token::{NewToken, Token};

// ── Managed state ────────────────────────────────────────────────────

/// Application state managed by Tauri.
pub struct AppState {
    /// The vault handle — `None` when locked / not yet created.
    pub vault: Mutex<Option<Vault>>,
    /// Persistent vault path (set once on create, reused on unlock).
    pub vault_path: Mutex<Option<String>>,
    /// Salts for key derivation (persisted alongside the vault).
    pub salts: Mutex<Option<VaultSalts>>,
    /// Cached token list (invalidated on mutation).
    pub token_cache: Mutex<Option<Vec<Token>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: Mutex::new(None),
            vault_path: Mutex::new(None),
            salts: Mutex::new(None),
            token_cache: Mutex::new(None),
        }
    }

    /// Invalidate the cached token list.
    fn invalidate_cache(&self) {
        if let Ok(mut cache) = self.token_cache.lock() {
            *cache = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSalts {
    pub sqlcipher_salt: [u8; 16],
    pub secret_salt: [u8; 16],
}

// ── KDF params (fast for development, production values in constants) ─

fn kdf_params() -> KdfParams {
    KdfParams {
        memory_kib: 65536,
        time_cost: 3,
        parallelism: 4,
    }
}

// ── Vault lifecycle ──────────────────────────────────────────────────

/// Create a brand-new encrypted vault.
///
/// Derives two independent keys (SQLCipher + secret encryption) from the
/// master password via Argon2id, creates the SQLCipher database, and
/// leaves the vault **unlocked**.
#[tauri::command]
pub fn vault_create(password: String, state: State<'_, AppState>) -> Result<String, String> {
    let sqlcipher_salt = generate_salt();
    let secret_salt = generate_salt();

    let (sqlcipher_key, secret_key) = keyforge_crypto::kdf::derive_key_pair(
        password.as_bytes(),
        &sqlcipher_salt,
        &secret_salt,
        &kdf_params(),
    )?;

    let vault_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.keyforge.app");
    std::fs::create_dir_all(&vault_dir)
        .map_err(|e| format!("Failed to create vault directory: {e}"))?;

    let vault_path = vault_dir.join("keyforge.vault");
    let vault_path_str = vault_path.to_string_lossy().to_string();

    let vault = Vault::create(&vault_path_str, &sqlcipher_key, secret_key)?;

    // Persist the salts next to the vault so we can re-derive on unlock.
    let salts = VaultSalts {
        sqlcipher_salt,
        secret_salt,
    };
    let salts_path = vault_dir.join("keyforge.salts");
    let salts_json =
        serde_json::to_vec(&salts).map_err(|e| format!("Failed to serialize salts: {e}"))?;
    std::fs::write(&salts_path, &salts_json).map_err(|e| format!("Failed to write salts: {e}"))?;

    *state.vault.lock().map_err(|e| e.to_string())? = Some(vault);
    *state.vault_path.lock().map_err(|e| e.to_string())? = Some(vault_path_str);
    *state.salts.lock().map_err(|e| e.to_string())? = Some(salts);
    state.invalidate_cache();

    Ok("vault_created".into())
}

/// Unlock the vault with the master password.
///
/// Re-derives keys from the stored salts and opens the existing SQLCipher
/// database.
#[tauri::command]
pub fn vault_unlock(password: String, state: State<'_, AppState>) -> Result<bool, String> {
    let vault_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.keyforge.app");

    let vault_path = vault_dir.join("keyforge.vault");
    if !vault_path.exists() {
        return Err("No vault found — create one first".into());
    }
    let vault_path_str = vault_path.to_string_lossy().to_string();

    // Load salts
    let salts_path = vault_dir.join("keyforge.salts");
    let salts_json =
        std::fs::read(&salts_path).map_err(|e| format!("Failed to read salts: {e}"))?;
    let salts: VaultSalts =
        serde_json::from_slice(&salts_json).map_err(|e| format!("Failed to parse salts: {e}"))?;

    let (sqlcipher_key, secret_key) = keyforge_crypto::kdf::derive_key_pair(
        password.as_bytes(),
        &salts.sqlcipher_salt,
        &salts.secret_salt,
        &kdf_params(),
    )?;

    let vault = Vault::open(&vault_path_str, &sqlcipher_key, secret_key)?;

    *state.vault.lock().map_err(|e| e.to_string())? = Some(vault);
    *state.vault_path.lock().map_err(|e| e.to_string())? = Some(vault_path_str);
    *state.salts.lock().map_err(|e| e.to_string())? = Some(salts);
    state.invalidate_cache();

    Ok(true)
}

/// Lock the vault (zeroize key from memory).
#[tauri::command]
pub fn vault_lock(state: State<'_, AppState>) -> Result<(), String> {
    // Dropping the Vault runs its Drop impl which zeroizes the secret key.
    *state.vault.lock().map_err(|e| e.to_string())? = None;
    state.invalidate_cache();
    Ok(())
}

/// Check whether the vault is currently locked.
#[tauri::command]
pub fn vault_is_locked(state: State<'_, AppState>) -> Result<bool, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    Ok(guard.is_none())
}

/// Check whether a vault file exists on disk.
#[tauri::command]
pub fn vault_exists() -> Result<bool, String> {
    let vault_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.keyforge.app");
    Ok(vault_dir.join("keyforge.vault").exists())
}

// ── Token CRUD ───────────────────────────────────────────────────────

/// List all tokens (cached when possible).
#[tauri::command]
pub fn token_list(state: State<'_, AppState>) -> Result<Vec<Token>, String> {
    // Return cache if available
    {
        let cache = state.token_cache.lock().map_err(|e| e.to_string())?;
        if let Some(ref tokens) = *cache {
            return Ok(tokens.clone());
        }
    }

    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    let tokens = vault.list_tokens()?;

    // Populate cache
    if let Ok(mut cache) = state.token_cache.lock() {
        *cache = Some(tokens.clone());
    }

    Ok(tokens)
}

#[derive(Debug, Deserialize)]
pub struct AddTokenInput {
    pub issuer: String,
    pub account: String,
    pub secret: String,
    pub algorithm: String,
    pub digits: u32,
    pub token_type: String,
    pub period: u32,
    pub counter: u64,
    pub icon: Option<String>,
}

/// Add a new token to the vault.
#[tauri::command]
pub fn token_add(input: AddTokenInput, state: State<'_, AppState>) -> Result<Token, String> {
    let secret_bytes =
        base32_decode(&input.secret).ok_or_else(|| "Invalid Base32 secret".to_string())?;

    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;

    let token = vault.add_token(NewToken {
        issuer: input.issuer,
        account: input.account,
        secret: secret_bytes,
        algorithm: input.algorithm,
        digits: input.digits,
        token_type: input.token_type,
        period: input.period,
        counter: input.counter,
        icon: input.icon,
    })?;

    // Invalidate cache after mutation.
    drop(guard);
    state.invalidate_cache();

    Ok(token)
}

/// Delete a token.
#[tauri::command]
pub fn token_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    vault.delete_token(&id)?;
    drop(guard);
    state.invalidate_cache();
    Ok(())
}

/// Update a token's issuer and account.
#[tauri::command]
pub fn token_update(
    id: String,
    issuer: String,
    account: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    vault.update_token(&id, &issuer, &account)?;
    drop(guard);
    state.invalidate_cache();
    Ok(())
}

/// Reorder tokens.
#[tauri::command]
pub fn token_reorder(ids: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    vault.reorder_tokens(&ids)?;
    drop(guard);
    state.invalidate_cache();
    Ok(())
}

/// Increment a HOTP counter and return the new value.
#[tauri::command]
pub fn token_increment_counter(id: String, state: State<'_, AppState>) -> Result<u64, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    let counter = vault.increment_counter(&id)?;
    drop(guard);
    state.invalidate_cache();
    Ok(counter)
}

// ── OTP generation ───────────────────────────────────────────────────

/// Generate a TOTP code for a stored token (secret retrieved from vault).
#[tauri::command]
pub fn otp_generate_totp(token_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;

    let token = vault.get_token(&token_id)?.ok_or("Token not found")?;
    let secret = vault.get_token_secret(&token_id)?;

    let algo = parse_algorithm(&token.algorithm)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let code =
        keyforge_crypto::totp::generate(&secret, now, token.period as u64, token.digits, algo);
    Ok(code)
}

/// Generate a TOTP code from a raw Base32 secret (for preview / manual entry).
#[tauri::command]
pub fn otp_generate_totp_raw(
    secret: String,
    algorithm: String,
    digits: u32,
    period: u64,
) -> Result<String, String> {
    let secret_bytes = base32_decode(&secret).ok_or_else(|| "Invalid Base32 secret".to_string())?;
    let algo = parse_algorithm(&algorithm)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let code = keyforge_crypto::totp::generate(&secret_bytes, now, period, digits, algo);
    Ok(code)
}

/// Generate a HOTP code for a stored token.
#[tauri::command]
pub fn otp_generate_hotp(token_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;

    let token = vault.get_token(&token_id)?.ok_or("Token not found")?;
    let secret = vault.get_token_secret(&token_id)?;

    let algo = parse_algorithm(&token.algorithm)?;

    let code = keyforge_crypto::hotp::generate(&secret, token.counter, token.digits, algo);
    Ok(code)
}

// ── Import / Export ──────────────────────────────────────────────────

/// Import tokens from `otpauth://` URIs.
#[tauri::command]
pub fn vault_import_uris(uris: Vec<String>, state: State<'_, AppState>) -> Result<usize, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    let count = vault.import_uris(&uris)?;
    drop(guard);
    state.invalidate_cache();
    Ok(count)
}

/// Export all tokens as `otpauth://` URIs.
#[tauri::command]
pub fn vault_export_uris(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    vault.export_uris()
}

/// Export all tokens as an encrypted file.
#[tauri::command]
pub fn vault_export_encrypted(
    export_password: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    vault.export_encrypted(export_password.as_bytes())
}

/// Import from an encrypted KeyForge export.
#[tauri::command]
pub fn vault_import_encrypted(
    data: Vec<u8>,
    password: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("Vault is locked")?;
    let count = vault.import_encrypted(&data, password.as_bytes())?;
    drop(guard);
    state.invalidate_cache();
    Ok(count)
}

// ── Platform info ────────────────────────────────────────────────────

/// Return basic platform information.
#[tauri::command]
pub fn platform_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    }))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn base32_decode(input: &str) -> Option<Vec<u8>> {
    base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &input.to_uppercase().replace([' ', '-'], ""),
    )
}

fn parse_algorithm(s: &str) -> Result<keyforge_crypto::hotp::Algorithm, String> {
    match s {
        "SHA1" => Ok(keyforge_crypto::hotp::Algorithm::SHA1),
        "SHA256" => Ok(keyforge_crypto::hotp::Algorithm::SHA256),
        "SHA512" => Ok(keyforge_crypto::hotp::Algorithm::SHA512),
        other => Err(format!("Unsupported algorithm: {other}")),
    }
}
