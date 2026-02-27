export interface Settings {
  autoLockTimeout: number;
  clipboardClearTimeout: number;
  biometricEnabled: boolean;
  compactMode: boolean;
  lockOnMinimize: boolean;
  lockOnBackground: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  autoLockTimeout: 60,
  clipboardClearTimeout: 30,
  biometricEnabled: false,
  compactMode: false,
  lockOnMinimize: true,
  lockOnBackground: true,
};
