import { DEFAULT_CLIPBOARD_CLEAR_MS } from '../constants/defaults';

let clearTimerId: ReturnType<typeof setTimeout> | null = null;

/**
 * Copy text to the clipboard and schedule automatic clearing.
 *
 * @param text  - The value to write (e.g. an OTP code).
 * @param clearAfterMs - Time in ms after which the clipboard is wiped
 *                        (defaults to DEFAULT_CLIPBOARD_CLEAR_MS).
 * @returns `true` if the write succeeded.
 */
export async function copyToClipboard(
  text: string,
  clearAfterMs: number = DEFAULT_CLIPBOARD_CLEAR_MS,
): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    scheduleClear(clearAfterMs);
    return true;
  } catch {
    return false;
  }
}

/**
 * Immediately clear the clipboard.
 */
export async function clearClipboard(): Promise<void> {
  cancelScheduledClear();
  try {
    await navigator.clipboard.writeText('');
  } catch {
    // Clipboard API may not be available in all contexts (e.g. non-secure origins).
  }
}

/** Cancel any pending auto-clear. */
export function cancelScheduledClear(): void {
  if (clearTimerId !== null) {
    clearTimeout(clearTimerId);
    clearTimerId = null;
  }
}

function scheduleClear(ms: number): void {
  cancelScheduledClear();
  clearTimerId = setTimeout(() => {
    clearTimerId = null;
    clearClipboard();
  }, ms);
}
