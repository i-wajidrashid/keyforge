import { describe, it, expect } from 'vitest';
import { generateTOTP, totpTimeRemaining } from '../totp/generate';

// RFC 6238 test secrets
const sha1Secret = new TextEncoder().encode('12345678901234567890');
const sha256Secret = new TextEncoder().encode('12345678901234567890123456789012');
const sha512Secret = new TextEncoder().encode('1234567890123456789012345678901234567890123456789012345678901234');

describe('TOTP (Web Crypto API)', () => {
  // RFC 6238 test vectors (8-digit codes)
  const testVectors: [number, 'SHA1' | 'SHA256' | 'SHA512', string, Uint8Array][] = [
    [59, 'SHA1', '94287082', sha1Secret],
    [59, 'SHA256', '46119246', sha256Secret],
    [59, 'SHA512', '90693936', sha512Secret],
    [1111111109, 'SHA1', '07081804', sha1Secret],
    [1111111109, 'SHA256', '68084774', sha256Secret],
    [1111111109, 'SHA512', '25091201', sha512Secret],
    [1111111111, 'SHA1', '14050471', sha1Secret],
    [1111111111, 'SHA256', '67062674', sha256Secret],
    [1111111111, 'SHA512', '99943326', sha512Secret],
    [1234567890, 'SHA1', '89005924', sha1Secret],
    [1234567890, 'SHA256', '91819424', sha256Secret],
    [1234567890, 'SHA512', '93441116', sha512Secret],
    [2000000000, 'SHA1', '69279037', sha1Secret],
    [2000000000, 'SHA256', '90698825', sha256Secret],
    [2000000000, 'SHA512', '38618901', sha512Secret],
    [20000000000, 'SHA1', '65353130', sha1Secret],
    [20000000000, 'SHA256', '77737706', sha256Secret],
    [20000000000, 'SHA512', '47863826', sha512Secret],
  ];

  it.each(testVectors)(
    'time=%i, algo=%s → %s',
    async (time, algorithm, expected, secret) => {
      const code = await generateTOTP(secret, time, 30, 8, algorithm);
      expect(code).toBe(expected);
    }
  );

  it('generates 6-digit code', async () => {
    const code = await generateTOTP(sha1Secret, 59, 30, 6, 'SHA1');
    expect(code).toHaveLength(6);
    expect(code).toBe('287082');
  });

  it('is deterministic', async () => {
    const code1 = await generateTOTP(sha1Secret, 1000, 30, 6, 'SHA1');
    const code2 = await generateTOTP(sha1Secret, 1000, 30, 6, 'SHA1');
    expect(code1).toBe(code2);
  });

  it('same period produces same code', async () => {
    const code1 = await generateTOTP(sha1Secret, 30, 30, 6, 'SHA1');
    const code2 = await generateTOTP(sha1Secret, 31, 30, 6, 'SHA1');
    expect(code1).toBe(code2);
  });
});

describe('totpTimeRemaining', () => {
  it('returns full period at boundary', () => {
    expect(totpTimeRemaining(0, 30)).toBe(30);
    expect(totpTimeRemaining(30, 30)).toBe(30);
  });

  it('returns correct remaining', () => {
    expect(totpTimeRemaining(1, 30)).toBe(29);
    expect(totpTimeRemaining(29, 30)).toBe(1);
  });
});
