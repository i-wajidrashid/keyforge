//! Typed error definitions for the crypto crate.

use std::fmt;

/// Errors that can occur during cryptographic operations.
#[derive(Debug)]
pub enum CryptoError {
    /// Failed to create an AES-256-GCM cipher instance.
    CipherInit(String),
    /// AES-256-GCM encryption failed.
    Encryption(String),
    /// Ciphertext is too short to contain a valid nonce + tag.
    CiphertextTooShort,
    /// AES-256-GCM decryption/authentication failed.
    DecryptionAuth,
    /// Argon2id parameter validation failed.
    InvalidKdfParams(String),
    /// Argon2id key derivation failed.
    KdfDerivation(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CipherInit(e) => write!(f, "Failed to create cipher: {}", e),
            Self::Encryption(e) => write!(f, "Encryption failed: {}", e),
            Self::CiphertextTooShort => write!(f, "Ciphertext too short"),
            Self::DecryptionAuth => write!(f, "Decryption failed: authentication error"),
            Self::InvalidKdfParams(e) => write!(f, "Invalid Argon2id params: {}", e),
            Self::KdfDerivation(e) => write!(f, "Argon2id derivation failed: {}", e),
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<CryptoError> for String {
    fn from(e: CryptoError) -> String {
        e.to_string()
    }
}
