/**
 * Lock / Unlock / Create-vault screen.
 *
 * Shows a password input and either "Unlock" or "Create Vault" depending
 * on whether a vault file already exists on disk.
 * Supports biometric authentication when available — stores the master
 * password in biometric-protected secure storage on first successful
 * unlock, then retrieves it on subsequent biometric unlocks.
 * Follows UI-SPEC.md: shake animation on wrong password, auto-focus.
 */

import {
  vaultCreate,
  vaultUnlock,
  vaultExists,
  biometryStatus,
  biometryHasPassword,
  biometryStorePassword,
  biometryGetPassword,
} from '../bridge';
import { ICON_FINGERPRINT } from '../icons';

export function renderLockScreen(
  root: HTMLElement,
  onUnlocked: () => void,
): void {
  root.innerHTML = `
    <div class="lock-screen">
      <div class="lock-content">
        <h1 class="lock-logo">KeyForge</h1>
        <p class="lock-tagline">Your keys, your devices.</p>

        <form id="lock-form" class="lock-form" autocomplete="off">
          <div class="input-group">
            <input
              id="password-input"
              type="password"
              class="input"
              placeholder="Master password"
              autocomplete="off"
              spellcheck="false"
              autofocus
            />
          </div>
          <button id="submit-btn" type="submit" class="btn btn-primary" disabled>
            Unlock
          </button>
        </form>

        <button id="bio-btn" class="btn-biometric" hidden title="Unlock with biometrics" aria-label="Unlock with biometrics">
          ${ICON_FINGERPRINT}
        </button>

        <p id="lock-error" class="lock-error" hidden></p>
        <p id="lock-status" class="lock-status"></p>
      </div>
    </div>
  `;

  const form = document.getElementById('lock-form') as HTMLFormElement;
  const input = document.getElementById('password-input') as HTMLInputElement;
  const btn = document.getElementById('submit-btn') as HTMLButtonElement;
  const bioBtn = document.getElementById('bio-btn') as HTMLButtonElement;
  const errorEl = document.getElementById('lock-error') as HTMLParagraphElement;
  const statusEl = document.getElementById('lock-status') as HTMLParagraphElement;

  let hasVault = false;
  let biometryAvailable = false;

  vaultExists()
    .then((exists) => {
      hasVault = exists;
      btn.textContent = exists ? 'Unlock' : 'Create Vault';
      statusEl.textContent = exists
        ? 'Enter your master password to unlock.'
        : 'Choose a master password to create your vault.';

      // Show biometric button when vault exists + biometry available + password stored
      if (exists) {
        biometryStatus()
          .then(async (status) => {
            if (status.isAvailable) {
              biometryAvailable = true;
              try {
                const stored = await biometryHasPassword();
                if (stored) {
                  bioBtn.hidden = false;
                }
              } catch { /* secure storage not available */ }
            }
          })
          .catch(() => { /* biometry not supported */ });
      }
    })
    .catch(() => {
      statusEl.textContent = 'Enter your master password.';
    });

  input.addEventListener('input', () => {
    btn.disabled = input.value.length === 0;
    errorEl.hidden = true;
  });

  // Biometric unlock — retrieves password from secure storage and unlocks
  bioBtn.addEventListener('click', async () => {
    errorEl.hidden = true;
    try {
      const password = await biometryGetPassword('Unlock KeyForge vault');
      btn.disabled = true;
      btn.textContent = 'Unlocking\u2026';
      await vaultUnlock(password);
      onUnlocked();
    } catch {
      errorEl.textContent = 'Biometric authentication failed';
      errorEl.hidden = false;
      btn.disabled = false;
      btn.textContent = 'Unlock';
    }
  });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!input.value) return;

    btn.disabled = true;
    btn.textContent = hasVault ? 'Unlocking\u2026' : 'Creating\u2026';
    errorEl.hidden = true;

    try {
      const password = input.value;
      if (hasVault) {
        await vaultUnlock(password);
      } else {
        await vaultCreate(password);
      }

      // Store password in biometric secure storage for future biometric unlocks
      if (biometryAvailable) {
        try {
          await biometryStorePassword(password);
        } catch { /* secure storage may not be available on all platforms */ }
      }

      // Clear password from DOM immediately after use
      input.value = '';
      onUnlocked();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);

      // Wrong password → shake animation per UI-SPEC.md
      input.classList.add('input-shake');
      setTimeout(() => input.classList.remove('input-shake'), 300);

      errorEl.textContent = msg.includes('Wrong password')
        ? 'Wrong password'
        : msg;
      errorEl.hidden = false;
      btn.disabled = false;
      btn.textContent = hasVault ? 'Unlock' : 'Create Vault';
      input.value = '';
      input.focus();
    }
  });
}
