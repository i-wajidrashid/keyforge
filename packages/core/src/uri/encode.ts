import type { Algorithm, TokenType } from '@keyforge/shared';
import {
  base32Encode,
  OTPAUTH_SCHEME,
  DEFAULT_ALGORITHM,
  DEFAULT_DIGITS,
  DEFAULT_PERIOD,
  DEFAULT_COUNTER,
} from '@keyforge/shared';

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

  if (params.algorithm && params.algorithm !== DEFAULT_ALGORITHM) {
    searchParams.set('algorithm', params.algorithm);
  }
  if (params.digits && params.digits !== DEFAULT_DIGITS) {
    searchParams.set('digits', params.digits.toString());
  }
  if (params.type === 'totp' && params.period && params.period !== DEFAULT_PERIOD) {
    searchParams.set('period', params.period.toString());
  }
  if (params.type === 'hotp') {
    searchParams.set('counter', (params.counter ?? DEFAULT_COUNTER).toString());
  }

  return `${OTPAUTH_SCHEME}${params.type}/${label}?${searchParams.toString()}`;
}
