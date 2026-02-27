import type { Algorithm, TokenType } from '@keyforge/shared';
import { base32Encode } from '@keyforge/shared';

export interface OtpUriParams {
  type: TokenType;
  issuer: string;
  account: string;
  secret: Uint8Array;
  algorithm?: Algorithm;
  digits?: number;
  period?: number;
  counter?: number;
}

/** Encode token parameters into an otpauth:// URI. */
export function encodeOtpUri(params: OtpUriParams): string {
  const secretBase32 = base32Encode(params.secret);
  const label = `${encodeURIComponent(params.issuer)}:${encodeURIComponent(params.account)}`;

  const searchParams = new URLSearchParams();
  searchParams.set('secret', secretBase32);
  searchParams.set('issuer', params.issuer);

  if (params.algorithm && params.algorithm !== 'SHA1') {
    searchParams.set('algorithm', params.algorithm);
  }
  if (params.digits && params.digits !== 6) {
    searchParams.set('digits', params.digits.toString());
  }
  if (params.type === 'totp' && params.period && params.period !== 30) {
    searchParams.set('period', params.period.toString());
  }
  if (params.type === 'hotp') {
    searchParams.set('counter', (params.counter ?? 0).toString());
  }

  return `otpauth://${params.type}/${label}?${searchParams.toString()}`;
}
