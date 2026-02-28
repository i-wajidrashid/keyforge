import type { Token, Algorithm, TokenType } from '@keyforge/shared';
import {
  base32Decode,
  OTPAUTH_SCHEME,
  DEFAULT_ALGORITHM,
  DEFAULT_DIGITS,
  DEFAULT_PERIOD,
  DEFAULT_COUNTER,
  DEFAULT_ISSUER,
  SUPPORTED_ALGORITHMS,
  SUPPORTED_DIGITS,
  ERR_INVALID_OTPAUTH_URI,
  ERR_UNKNOWN_TOKEN_TYPE,
  ERR_MISSING_SECRET,
} from '@keyforge/shared';

export interface ParsedOtpUri {
  type: TokenType;
  issuer: string;
  account: string;
  secret: Uint8Array;
  secretBase32: string;
  algorithm: Algorithm;
  digits: number;
  period: number;
  counter: number;
}

/** Parse an otpauth:// URI into its component parts. */
export function parseOtpUri(uri: string): ParsedOtpUri {
  if (!uri.startsWith(OTPAUTH_SCHEME)) {
    throw new Error(ERR_INVALID_OTPAUTH_URI);
  }

  const url = new URL(uri);
  const type = url.hostname as TokenType;

  if (type !== 'totp' && type !== 'hotp') {
    throw new Error(ERR_UNKNOWN_TOKEN_TYPE(type));
  }

  // Parse label
  const label = decodeURIComponent(url.pathname.slice(1));
  let issuerFromLabel: string | undefined;
  let account: string;

  const colonIndex = label.indexOf(':');
  if (colonIndex !== -1) {
    issuerFromLabel = label.slice(0, colonIndex);
    account = label.slice(colonIndex + 1).trim();
  } else {
    account = label;
  }

  // Extract params
  const secretParam = url.searchParams.get('secret');
  if (!secretParam) {
    throw new Error(ERR_MISSING_SECRET);
  }

  const secretBase32 = secretParam.toUpperCase();
  const secret = base32Decode(secretBase32);

  const issuer = url.searchParams.get('issuer') || issuerFromLabel || DEFAULT_ISSUER;
  const rawAlgorithm = (url.searchParams.get('algorithm')?.toUpperCase() || DEFAULT_ALGORITHM) as Algorithm;
  const rawDigits = parseInt(url.searchParams.get('digits') || String(DEFAULT_DIGITS), 10);
  const rawPeriod = parseInt(url.searchParams.get('period') || String(DEFAULT_PERIOD), 10);
  const rawCounter = parseInt(url.searchParams.get('counter') || String(DEFAULT_COUNTER), 10);

  // Validate algorithm
  const algorithm: Algorithm = (SUPPORTED_ALGORITHMS as readonly string[]).includes(rawAlgorithm)
    ? rawAlgorithm
    : DEFAULT_ALGORITHM as Algorithm;

  // Validate digits
  const digits: number = (SUPPORTED_DIGITS as readonly number[]).includes(rawDigits)
    ? rawDigits
    : DEFAULT_DIGITS;

  // Validate period (must be positive)
  const period: number = Number.isFinite(rawPeriod) && rawPeriod > 0 ? rawPeriod : DEFAULT_PERIOD;

  // Validate counter (must be non-negative)
  const counter: number = Number.isFinite(rawCounter) && rawCounter >= 0 ? rawCounter : DEFAULT_COUNTER;

  return {
    type,
    issuer,
    account,
    secret,
    secretBase32,
    algorithm,
    digits,
    period,
    counter,
  };
}
