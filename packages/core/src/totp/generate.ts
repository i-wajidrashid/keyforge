import type { Algorithm } from '@keyforge/shared';

/**
 * Generate a TOTP code using Web Crypto API (for Chrome extension).
 * On Tauri platforms, this should be replaced with Rust calls.
 */
export async function generateTOTP(
  secret: Uint8Array,
  time: number,
  period: number,
  digits: number,
  algorithm: Algorithm
): Promise<string> {
  const counter = Math.floor(time / period);
  return generateHOTPCode(secret, counter, digits, algorithm);
}

/**
 * Internal: Generate HMAC-based OTP code using Web Crypto API.
 */
async function generateHOTPCode(
  secret: Uint8Array,
  counter: number,
  digits: number,
  algorithm: Algorithm
): Promise<string> {
  const hashAlgorithm = getHashAlgorithm(algorithm);

  // Import key for HMAC
  const key = await crypto.subtle.importKey(
    'raw',
    secret,
    { name: 'HMAC', hash: hashAlgorithm },
    false,
    ['sign']
  );

  // Counter to 8-byte big-endian buffer
  const counterBuffer = new ArrayBuffer(8);
  const counterView = new DataView(counterBuffer);
  counterView.setBigUint64(0, BigInt(counter));

  // HMAC sign
  const hmacResult = new Uint8Array(
    await crypto.subtle.sign('HMAC', key, counterBuffer)
  );

  // Dynamic truncation (RFC 4226 Section 5.4)
  const offset = hmacResult[hmacResult.length - 1] & 0x0f;
  const binary =
    ((hmacResult[offset] & 0x7f) << 24) |
    ((hmacResult[offset + 1]) << 16) |
    ((hmacResult[offset + 2]) << 8) |
    hmacResult[offset + 3];

  const otp = binary % Math.pow(10, digits);
  return otp.toString().padStart(digits, '0');
}

function getHashAlgorithm(algorithm: Algorithm): string {
  switch (algorithm) {
    case 'SHA1': return 'SHA-1';
    case 'SHA256': return 'SHA-256';
    case 'SHA512': return 'SHA-512';
    default: return 'SHA-1';
  }
}

/**
 * Get the number of seconds remaining in the current TOTP period.
 */
export function totpTimeRemaining(time: number, period: number): number {
  return period - (time % period);
}
