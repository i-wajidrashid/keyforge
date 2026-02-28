import { describe, it, expect } from 'vitest';
import { generateHOTP } from '../hotp/generate';

describe('HOTP (Web Crypto API)', () => {
  const secret = new TextEncoder().encode('12345678901234567890');

  // RFC 4226 Appendix D test vectors
  const expected = [
    '755224', '287082', '359152', '969429', '338314',
    '254676', '287922', '162583', '399871', '520489',
  ];

  it.each(expected.map((code, i) => [i, code]))(
    'counter=%i → %s',
    async (counter, expectedCode) => {
      const code = await generateHOTP(secret, counter as number, 6, 'SHA1');
      expect(code).toBe(expectedCode);
    }
  );

  it('generates 8-digit code', async () => {
    const code = await generateHOTP(secret, 0, 8, 'SHA1');
    expect(code).toHaveLength(8);
  });

  it('is deterministic', async () => {
    const code1 = await generateHOTP(secret, 42, 6, 'SHA1');
    const code2 = await generateHOTP(secret, 42, 6, 'SHA1');
    expect(code1).toBe(code2);
  });
});
