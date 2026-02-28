export interface Token {
  id: string;
  issuer: string;
  account: string;
  algorithm: Algorithm;
  digits: number;
  type: TokenType;
  period: number;
  counter: number;
  icon?: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
  lastModified?: string;
  deviceId?: string;
  syncVersion?: number;
}

export type TokenType = 'totp' | 'hotp';
export type Algorithm = 'SHA1' | 'SHA256' | 'SHA512';
