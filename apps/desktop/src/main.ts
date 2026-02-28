/**
 * KeyForge desktop application — frontend entry point.
 *
 * Bootstraps the UI and routes between lock screen ↔ token list
 * based on vault state.  All crypto and storage happens in Rust
 * via Tauri commands (see src-tauri/src/commands.rs).
 */

import './styles/app.css';
import { renderLockScreen } from './screens/lock';
import { renderTokenList } from './screens/tokens';
import { vaultIsLocked, vaultLock } from './bridge';

const app = document.getElementById('app')!;

/** Show the lock screen. */
function showLock(): void {
  renderLockScreen(app, showTokens);
}

/** Show the main token list. */
function showTokens(): void {
  renderTokenList(app, showLock);
}

/** Lock the vault when the app goes to background / is minimized. */
document.addEventListener('visibilitychange', async () => {
  if (document.visibilityState === 'hidden') {
    try {
      const locked = await vaultIsLocked();
      if (!locked) {
        await vaultLock();
        showLock();
      }
    } catch { /* already locked or not ready */ }
  }
});

/** Check vault state and render the right screen. */
async function boot(): Promise<void> {
  try {
    const locked = await vaultIsLocked();
    if (locked) {
      showLock();
    } else {
      showTokens();
    }
  } catch {
    // Tauri not ready yet or error — show lock screen
    showLock();
  }
}

boot();
