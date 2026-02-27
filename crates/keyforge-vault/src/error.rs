//! Typed error definitions for the vault crate.

use std::fmt;

/// Errors that can occur during vault operations.
#[derive(Debug)]
pub enum VaultError {
    /// Failed to open or create the SQLite database file.
    DatabaseOpen(String),
    /// SQLCipher key could not be set.
    SetEncryptionKey(String),
    /// Database decryption failed — wrong password or corruption.
    WrongPasswordOrCorrupted,
    /// A database migration failed.
    Migration(String),
    /// Failed to read the current schema version.
    SchemaVersion(String),
    /// Failed to encrypt a token secret.
    EncryptSecret(String),
    /// Failed to decrypt a token secret.
    DecryptSecret(String),
    /// A database query or statement failed.
    Query(String),
    /// The requested token was not found.
    TokenNotFound,
    /// An imported/exported file was structurally invalid.
    InvalidExportFile,
    /// Serialization/deserialization failed.
    Serialization(String),
    /// An `otpauth://` URI was malformed.
    InvalidUri(String),
    /// A required URI parameter is missing.
    MissingUriParam(&'static str),
    /// The base32-encoded secret in a URI was invalid.
    InvalidBase32Secret,
    /// An unknown OTP token type was encountered.
    UnknownTokenType(String),
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseOpen(e) => write!(f, "Failed to open vault database: {}", e),
            Self::SetEncryptionKey(e) => write!(f, "Failed to set encryption key: {}", e),
            Self::WrongPasswordOrCorrupted => write!(f, "Wrong password or corrupted vault"),
            Self::Migration(e) => write!(f, "Migration failed: {}", e),
            Self::SchemaVersion(e) => write!(f, "Failed to read schema version: {}", e),
            Self::EncryptSecret(e) => write!(f, "Failed to encrypt secret: {}", e),
            Self::DecryptSecret(e) => write!(f, "Failed to decrypt secret: {}", e),
            Self::Query(e) => write!(f, "Database query failed: {}", e),
            Self::TokenNotFound => write!(f, "Token not found"),
            Self::InvalidExportFile => write!(f, "Invalid export file"),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::InvalidUri(detail) => write!(f, "Invalid otpauth URI: {}", detail),
            Self::MissingUriParam(name) => write!(f, "Missing URI parameter: {}", name),
            Self::InvalidBase32Secret => write!(f, "Invalid base32 secret"),
            Self::UnknownTokenType(t) => write!(f, "Unknown token type: {}", t),
        }
    }
}

impl std::error::Error for VaultError {}

// Convenience conversion so callers using `Result<_, String>` still work
// during the migration period.
impl From<VaultError> for String {
    fn from(e: VaultError) -> String {
        e.to_string()
    }
}
