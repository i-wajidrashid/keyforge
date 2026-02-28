import { describe, it, expect } from 'vitest';
import { parseOtpUri } from '../uri/parse';
import { encodeOtpUri } from '../uri/encode';

describe('parseOtpUri', () => {
  it('parses basic TOTP URI', () => {
    const result = parseOtpUri('otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&algorithm=SHA1&digits=6&period=30');
    expect(result.type).toBe('totp');
    expect(result.issuer).toBe('GitHub');
    expect(result.account).toBe('user@example.com');
    expect(result.algorithm).toBe('SHA1');
    expect(result.digits).toBe(6);
    expect(result.period).toBe(30);
  });

  it('parses HOTP URI with counter', () => {
    const result = parseOtpUri('otpauth://hotp/Test:user?secret=JBSWY3DPEHPK3PXP&counter=42');
    expect(result.type).toBe('hotp');
    expect(result.counter).toBe(42);
  });

  it('applies defaults for missing params', () => {
    const result = parseOtpUri('otpauth://totp/user?secret=JBSWY3DPEHPK3PXP');
    expect(result.issuer).toBe('Unknown');
    expect(result.algorithm).toBe('SHA1');
    expect(result.digits).toBe(6);
    expect(result.period).toBe(30);
  });

  it('uses issuer from query over label', () => {
    const result = parseOtpUri('otpauth://totp/LabelIssuer:user?secret=JBSWY3DPEHPK3PXP&issuer=QueryIssuer');
    expect(result.issuer).toBe('QueryIssuer');
  });

  it('throws on invalid URI', () => {
    expect(() => parseOtpUri('https://example.com')).toThrow();
  });

  it('throws on missing secret', () => {
    expect(() => parseOtpUri('otpauth://totp/Test?algorithm=SHA1')).toThrow();
  });

  it('throws on unknown type', () => {
    expect(() => parseOtpUri('otpauth://unknown/Test?secret=JBSWY3DPEHPK3PXP')).toThrow();
  });
});

describe('encodeOtpUri', () => {
  it('encodes basic TOTP URI', () => {
    const uri = encodeOtpUri({
      type: 'totp',
      issuer: 'GitHub',
      account: 'user@example.com',
      secret: new TextEncoder().encode('Hello!\xDEX\xD7\x8E'),
    });
    expect(uri).toContain('otpauth://totp/');
    expect(uri).toContain('GitHub');
    expect(uri).toContain('secret=');
  });

  it('round-trips through parse', () => {
    const original = {
      type: 'totp' as const,
      issuer: 'GitHub',
      account: 'user@example.com',
      secret: new TextEncoder().encode('testsecret12345678'),
      algorithm: 'SHA256' as const,
      digits: 8,
      period: 60,
    };
    const uri = encodeOtpUri(original);
    const parsed = parseOtpUri(uri);
    
    expect(parsed.type).toBe(original.type);
    expect(parsed.issuer).toBe(original.issuer);
    expect(parsed.account).toBe(original.account);
    expect(parsed.algorithm).toBe(original.algorithm);
    expect(parsed.digits).toBe(original.digits);
    expect(parsed.period).toBe(original.period);
  });
});
