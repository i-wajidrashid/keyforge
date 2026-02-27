import type { Algorithm } from '@keyforge/shared';
import { COUNTER_BYTE_LENGTH, HASH_ALGORITHM_MAP } from '@keyforge/shared';

/** Generate a TOTP code using Web Crypto API (Chrome extension fallback). */
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

async function generateHOTPCode(
  secret: Uint8Array,
  counter: number,
  digits: number,
  algorithm: Algorithm
): Promise<string> {
  const hashAlgorithm = HASH_ALGORITHM_MAP[algorithm] ?? HASH_ALGORITHM_MAP.SHA1;

  const keyData = new Uint8Array(secret).buffer;
  const key = await crypto.subtle.importKey(
    'raw',
    keyData as ArrayBuffer,
    { name: 'HMAC', hash: hashAlgorithm },
    false,
    ['sign']
  );

  const counterBuffer = new ArrayBuffer(COUNTER_BYTE_LENGTH);
  const counterView = new DataView(counterBuffer);
  counterView.setBigUint64(0, BigInt(counter));

  const hmacResult = new Uint8Array(
    await crypto.subtle.sign('HMAC', key, counterBuffer)
  );

  // Dynamic truncation (RFC 4226 §5.4)
  const offset = hmacResult[hmacResult.length - 1] & 0x0f;
  const binary =
    ((hmacResult[offset] & 0x7f) << 24) |
    ((hmacResult[offset + 1]) << 16) |
    ((hmacResult[offset + 2]) << 8) |
    hmacResult[offset + 3];

  const otp = binary % Math.pow(10, digits);
  return otp.toString().padStart(digits, '0');
}

/**
 * Get the number of seconds remaining in the current TOTP period.
 */
export function totpTimeRemaining(time: number, period: number): number {
  return period - (time % period);
}
