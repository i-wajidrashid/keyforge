//! Key derivation using Argon2id

use argon2::{Algorithm, Argon2, Params, Version};

/// Default Argon2id parameters per SECURITY.md
pub const DEFAULT_MEMORY_KIB: u32 = 65536; // 64 MiB
pub const DEFAULT_TIME_COST: u32 = 3;
pub const DEFAULT_PARALLELISM: u32 = 4;
pub const KEY_LENGTH: usize = 32; // 256-bit key

/// KDF parameters
#[derive(Debug, Clone)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_kib: DEFAULT_MEMORY_KIB,
            time_cost: DEFAULT_TIME_COST,
            parallelism: DEFAULT_PARALLELISM,
        }
    }
}

/// Derive a 256-bit key from a password using Argon2id
///
/// # Arguments
/// * `password` - The master password
/// * `salt` - 16-byte random salt
/// * `params` - Argon2id parameters
///
/// # Returns
/// 32-byte derived key
pub fn derive_key(
    password: &[u8],
    salt: &[u8; 16],
    params: &KdfParams,
) -> Result<[u8; KEY_LENGTH], String> {
    let argon2_params = Params::new(
        params.memory_kib,
        params.time_cost,
        params.parallelism,
        Some(KEY_LENGTH),
    )
    .map_err(|e| format!("Invalid Argon2id params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut output = [0u8; KEY_LENGTH];
    argon2
        .hash_password_into(password, salt, &mut output)
        .map_err(|e| format!("Argon2id derivation failed: {}", e))?;

    Ok(output)
}

/// Derive two separate keys from a password (one for SQLCipher, one for secret encryption)
///
/// Uses different salts to ensure key independence
pub fn derive_key_pair(
    password: &[u8],
    sqlcipher_salt: &[u8; 16],
    secret_salt: &[u8; 16],
    params: &KdfParams,
) -> Result<([u8; KEY_LENGTH], [u8; KEY_LENGTH]), String> {
    let sqlcipher_key = derive_key(password, sqlcipher_salt, params)?;
    let secret_key = derive_key(password, secret_salt, params)?;
    Ok((sqlcipher_key, secret_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use reduced params for testing to keep tests fast
    fn test_params() -> KdfParams {
        KdfParams {
            memory_kib: 1024, // 1 MiB for fast tests
            time_cost: 1,
            parallelism: 1,
        }
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"test-password";
        let salt = [1u8; 16];
        let params = test_params();

        let key1 = derive_key(password, &salt, &params).unwrap();
        let key2 = derive_key(password, &salt, &params).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_length() {
        let password = b"test-password";
        let salt = [1u8; 16];
        let params = test_params();

        let key = derive_key(password, &salt, &params).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_different_salts_different_keys() {
        let password = b"test-password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        let params = test_params();

        let key1 = derive_key(password, &salt1, &params).unwrap();
        let key2 = derive_key(password, &salt2, &params).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let salt = [1u8; 16];
        let params = test_params();

        let key1 = derive_key(b"password1", &salt, &params).unwrap();
        let key2 = derive_key(b"password2", &salt, &params).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_pair() {
        let password = b"test-password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        let params = test_params();

        let (key_a, key_b) = derive_key_pair(password, &salt1, &salt2, &params).unwrap();

        assert_eq!(key_a.len(), 32);
        assert_eq!(key_b.len(), 32);
        assert_ne!(key_a, key_b);
    }

    #[test]
    fn test_empty_password() {
        let salt = [1u8; 16];
        let params = test_params();

        let result = derive_key(b"", &salt, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_password() {
        let password = vec![b'a'; 1024];
        let salt = [1u8; 16];
        let params = test_params();

        let result = derive_key(&password, &salt, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }
}
