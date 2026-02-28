import { DEFAULT_AUTO_LOCK_TIMEOUT, DEFAULT_CLIPBOARD_CLEAR_TIMEOUT } from '../constants/defaults';

export interface Settings {
  autoLockTimeout: number;
  clipboardClearTimeout: number;
  biometricEnabled: boolean;
  compactMode: boolean;
  lockOnMinimize: boolean;
  lockOnBackground: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  autoLockTimeout: DEFAULT_AUTO_LOCK_TIMEOUT,
  clipboardClearTimeout: DEFAULT_CLIPBOARD_CLEAR_TIMEOUT,
  biometricEnabled: false,
  compactMode: false,
  lockOnMinimize: true,
  lockOnBackground: true,
};
