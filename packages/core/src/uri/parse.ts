import type { Token, Algorithm, TokenType } from '@keyforge/shared';
import { base32Decode } from '@keyforge/shared';

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

/**
 * Parse an otpauth:// URI into its component parts.
 */
export function parseOtpUri(uri: string): ParsedOtpUri {
  if (!uri.startsWith('otpauth://')) {
    throw new Error('Invalid otpauth URI: must start with otpauth://');
  }

  const url = new URL(uri);
  const type = url.hostname as TokenType;
  
  if (type !== 'totp' && type !== 'hotp') {
    throw new Error(`Unknown token type: ${type}`);
  }

  // Parse label (path without leading slash)
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
    throw new Error('Missing secret parameter');
  }

  const secretBase32 = secretParam.toUpperCase();
  const secret = base32Decode(secretBase32);

  const issuer = url.searchParams.get('issuer') || issuerFromLabel || 'Unknown';
  const algorithm = (url.searchParams.get('algorithm')?.toUpperCase() || 'SHA1') as Algorithm;
  const digits = parseInt(url.searchParams.get('digits') || '6', 10);
  const period = parseInt(url.searchParams.get('period') || '30', 10);
  const counter = parseInt(url.searchParams.get('counter') || '0', 10);

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
