//! Shared constants for the vault crate.

/// The `otpauth://` URI scheme prefix.
pub const OTPAUTH_SCHEME: &str = "otpauth://";
/// Length of the `otpauth://` scheme prefix.
pub const OTPAUTH_SCHEME_LEN: usize = OTPAUTH_SCHEME.len();

/// Salt size in bytes for encrypted exports.
pub const EXPORT_SALT_SIZE: usize = 16;

/// Default issuer when the URI does not specify one.
pub const DEFAULT_ISSUER: &str = "Unknown";
/// Default HMAC algorithm.
pub const DEFAULT_ALGORITHM: &str = "SHA1";
/// Default number of OTP digits.
pub const DEFAULT_DIGITS: u32 = 6;
/// Default TOTP period in seconds.
pub const DEFAULT_PERIOD: u32 = 30;
/// Default HOTP counter.
pub const DEFAULT_COUNTER: u64 = 0;

/// Supported OTP token types.
pub const TOKEN_TYPE_TOTP: &str = "totp";
pub const TOKEN_TYPE_HOTP: &str = "hotp";

/// Initial sort-order sentinel (no tokens exist yet).
pub const INITIAL_SORT_ORDER: i32 = -1;

/// Current schema version.
pub const SCHEMA_VERSION: i32 = 1;
