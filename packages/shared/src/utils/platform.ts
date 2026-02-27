import type { Platform } from '../types/platform';

/**
 * Detect the current runtime platform.
 */
export function detectPlatform(): Platform {
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    // Check if mobile or desktop
    if (typeof navigator !== 'undefined' && /Android|iPhone|iPad/i.test(navigator.userAgent)) {
      return 'tauri-mobile';
    }
    return 'tauri-desktop';
  }

  if (typeof chrome !== 'undefined' && chrome?.runtime?.id) {
    return 'extension';
  }

  return 'web';
}

/**
 * Check if the current runtime has access to Rust via Tauri.
 */
export function hasTauriBridge(): boolean {
  const platform = detectPlatform();
  return platform === 'tauri-desktop' || platform === 'tauri-mobile';
}
