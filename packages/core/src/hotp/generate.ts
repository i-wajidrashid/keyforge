import type { Algorithm } from '@keyforge/shared';

/**
 * Generate an HOTP code using Web Crypto API (for Chrome extension).
 * On Tauri platforms, this should be replaced with Rust calls.
 */
export async function generateHOTP(
  secret: Uint8Array,
  counter: number,
  digits: number,
  algorithm: Algorithm
): Promise<string> {
  const hashAlgorithm = getHashAlgorithm(algorithm);

  const keyData = new Uint8Array(secret).buffer;
  const key = await crypto.subtle.importKey(
    'raw',
    keyData as ArrayBuffer,
    { name: 'HMAC', hash: hashAlgorithm },
    false,
    ['sign']
  );

  const counterBuffer = new ArrayBuffer(8);
  const counterView = new DataView(counterBuffer);
  counterView.setBigUint64(0, BigInt(counter));

  const hmacResult = new Uint8Array(
    await crypto.subtle.sign('HMAC', key, counterBuffer)
  );

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
