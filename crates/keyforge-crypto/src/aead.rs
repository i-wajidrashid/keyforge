//! AES-256-GCM authenticated encryption

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

pub const NONCE_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;

/// Encrypt plaintext using AES-256-GCM
///
/// Returns: [12 bytes nonce][N bytes ciphertext][16 bytes GCM tag]
/// The nonce is randomly generated and prepended to the output.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let nonce_bytes = crate::random::generate_bytes(NONCE_SIZE);
    encrypt_with_nonce(plaintext, key, &nonce_bytes)
}

/// Encrypt with a specific nonce (for testing)
pub fn encrypt_with_nonce(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("Failed to create cipher: {}", e))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Output format: [nonce][ciphertext+tag]
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt ciphertext that was encrypted with `encrypt`
///
/// Input format: [12 bytes nonce][N bytes ciphertext][16 bytes GCM tag]
pub fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_SIZE + TAG_SIZE {
        return Err("Ciphertext too short".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("Failed to create cipher: {}", e))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: authentication error".to_string())?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_format() {
        let key = test_key();
        let plaintext = b"test";

        let encrypted = encrypt(plaintext, &key).unwrap();

        // Should be: 12 (nonce) + 4 (plaintext) + 16 (tag) = 32 bytes
        assert_eq!(encrypted.len(), NONCE_SIZE + plaintext.len() + TAG_SIZE);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let plaintext = b"secret data";

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"secret data";

        let mut encrypted = encrypt(plaintext, &key).unwrap();
        // Tamper with a byte in the ciphertext
        let mid = encrypted.len() / 2;
        encrypted[mid] ^= 0xff;

        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let key = test_key();
        let plaintext = b"";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let key = test_key();
        let plaintext = vec![0xABu8; 1_000_000]; // 1 MB

        let encrypted = encrypt(&plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ciphertext_too_short() {
        let key = test_key();
        let short = vec![0u8; NONCE_SIZE + TAG_SIZE - 1];

        let result = decrypt(&short, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_nonces_different_ciphertext() {
        let key = test_key();
        let plaintext = b"same plaintext";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        // Different random nonces should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(decrypt(&encrypted1, &key).unwrap(), plaintext);
        assert_eq!(decrypt(&encrypted2, &key).unwrap(), plaintext);
    }
}
