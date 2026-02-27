export interface VaultMeta {
  schemaVersion: number;
  vaultCreatedAt: string;
  lastLockedAt?: string;
  deviceId?: string;
}

export type VaultState = 'locked' | 'unlocked' | 'uninitialized';
