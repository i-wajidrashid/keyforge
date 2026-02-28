import { describe, it, expect } from 'vitest';
import { base32Decode, base32Encode } from '../utils/base32';

describe('base32', () => {
  it('decodes standard test vectors', () => {
    // "Hello!" in base32 is "JBSWY3DPEE"
    const decoded = base32Decode('JBSWY3DPEE');
    expect(new TextDecoder().decode(decoded)).toBe('Hello!');
  });

  it('encodes standard test vectors', () => {
    const encoded = base32Encode(new TextEncoder().encode('Hello!'));
    expect(encoded).toBe('JBSWY3DPEE');
  });

  it('round-trips', () => {
    const original = new TextEncoder().encode('12345678901234567890');
    const encoded = base32Encode(original);
    const decoded = base32Decode(encoded);
    expect(decoded).toEqual(original);
  });

  it('handles empty input', () => {
    expect(base32Decode('')).toEqual(new Uint8Array(0));
    expect(base32Encode(new Uint8Array(0))).toBe('');
  });

  it('strips spaces and dashes', () => {
    const withSpaces = base32Decode('JBSW Y3DP EE');
    const clean = base32Decode('JBSWY3DPEE');
    expect(withSpaces).toEqual(clean);
  });

  it('strips padding', () => {
    const withPadding = base32Decode('JBSWY3DPEE======');
    const clean = base32Decode('JBSWY3DPEE');
    expect(withPadding).toEqual(clean);
  });

  it('throws on invalid characters', () => {
    expect(() => base32Decode('INVALID!@#')).toThrow();
  });

  it('is case-insensitive', () => {
    const upper = base32Decode('JBSWY3DPEE');
    const lower = base32Decode('jbswy3dpee');
    expect(upper).toEqual(lower);
  });

  it('decodes JBSWY3DPEHPK3PXP (common test secret)', () => {
    const decoded = base32Decode('JBSWY3DPEHPK3PXP');
    // "Hello!" + 0xDEADBEEF
    expect(Array.from(decoded)).toEqual([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x21, 0xde, 0xad, 0xbe, 0xef]);
  });
});
