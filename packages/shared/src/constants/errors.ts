/** Centralized error messages for the KeyForge TypeScript packages. */

// URI parsing
export const ERR_INVALID_OTPAUTH_URI = 'Invalid otpauth URI: must start with otpauth://';
export const ERR_UNKNOWN_TOKEN_TYPE = (type: string) => `Unknown token type: ${type}`;
export const ERR_MISSING_SECRET = 'Missing secret parameter';

// Base32
export const ERR_INVALID_BASE32_CHAR = (char: string) => `Invalid base32 character: ${char}`;

// Clipboard
export const ERR_CLIPBOARD_UNAVAILABLE = 'Clipboard API is not available';
