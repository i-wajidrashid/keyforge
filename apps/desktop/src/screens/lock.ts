/**
 * Lock / Unlock / Create-vault screen.
 *
 * Shows a password input and either "Unlock" or "Create Vault" depending
 * on whether a vault file already exists on disk.
 */

import { vaultCreate, vaultUnlock, vaultExists } from '../bridge';

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

        <p id="lock-error" class="lock-error" hidden></p>
        <p id="lock-status" class="lock-status"></p>
      </div>
    </div>
  `;

  const form = document.getElementById('lock-form') as HTMLFormElement;
  const input = document.getElementById('password-input') as HTMLInputElement;
  const btn = document.getElementById('submit-btn') as HTMLButtonElement;
  const errorEl = document.getElementById('lock-error') as HTMLParagraphElement;
  const statusEl = document.getElementById('lock-status') as HTMLParagraphElement;

  let hasVault = false;

  // Check if vault exists and update button label
  vaultExists()
    .then((exists) => {
      hasVault = exists;
      btn.textContent = exists ? 'Unlock' : 'Create Vault';
      statusEl.textContent = exists
        ? 'Enter your master password to unlock.'
        : 'Choose a master password to create your vault.';
    })
    .catch(() => {
      statusEl.textContent = 'Enter your master password.';
    });

  // Enable button when password is non-empty
  input.addEventListener('input', () => {
    btn.disabled = input.value.length === 0;
    errorEl.hidden = true;
  });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const password = input.value;
    if (!password) return;

    btn.disabled = true;
    btn.textContent = hasVault ? 'Unlocking…' : 'Creating…';
    errorEl.hidden = true;

    try {
      if (hasVault) {
        await vaultUnlock(password);
      } else {
        await vaultCreate(password);
      }
      onUnlocked();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      errorEl.textContent = msg.includes('Wrong password')
        ? 'Wrong password. Try again.'
        : msg;
      errorEl.hidden = false;
      btn.disabled = false;
      btn.textContent = hasVault ? 'Unlock' : 'Create Vault';
      input.focus();
      input.select();
    }
  });
}
