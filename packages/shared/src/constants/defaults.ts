export const DEFAULT_PERIOD = 30;
export const DEFAULT_DIGITS = 6;
export const DEFAULT_ALGORITHM = 'SHA1' as const;
export const DEFAULT_TOKEN_TYPE = 'totp' as const;
export const DEFAULT_COUNTER = 0;
export const DEFAULT_ISSUER = 'Unknown';
export const DEFAULT_AUTO_LOCK_TIMEOUT = 60;
export const DEFAULT_CLIPBOARD_CLEAR_TIMEOUT = 30;

/** Milliseconds per second, used for epoch conversions. */
export const MS_PER_SECOND = 1000;
/** TOTP timer tick interval in milliseconds. */
export const TIMER_INTERVAL_MS = 1000;
/** The clipboard auto-clear timeout in milliseconds (derived from seconds). */
export const DEFAULT_CLIPBOARD_CLEAR_MS = DEFAULT_CLIPBOARD_CLEAR_TIMEOUT * MS_PER_SECOND;

/** The `otpauth://` URI scheme prefix. */
export const OTPAUTH_SCHEME = 'otpauth://';
/** Byte length of an HMAC counter buffer (8 bytes / 64-bit big-endian). */
export const COUNTER_BYTE_LENGTH = 8;

/** Web Crypto algorithm name mapping from OTP algorithm identifiers. */
export const HASH_ALGORITHM_MAP = {
  SHA1: 'SHA-1',
  SHA256: 'SHA-256',
  SHA512: 'SHA-512',
} as const;
