/**
 * Add Token screen — manual entry form.
 *
 * Lets the user type in issuer, account, Base32 secret, and optional
 * settings (algorithm, digits, period).  Validates input and calls
 * the Tauri `token_add` command.
 */

import { tokenAdd } from '../bridge';

export function renderAddTokenScreen(
  root: HTMLElement,
  onDone: () => void,
): void {
  root.innerHTML = `
    <div class="app-shell">
      <header class="app-header">
        <button id="back-btn" class="btn-icon" title="Back" aria-label="Back">←</button>
        <h1 class="app-title">Add Token</h1>
        <div class="header-actions"></div>
      </header>
      <main class="add-form-container">
        <form id="add-form" class="add-form" autocomplete="off">
          <div class="form-field">
            <label for="add-issuer">Issuer</label>
            <input id="add-issuer" type="text" class="input" placeholder="GitHub, AWS, Google…" required />
          </div>

          <div class="form-field">
            <label for="add-account">Account</label>
            <input id="add-account" type="text" class="input" placeholder="user@example.com" />
          </div>

          <div class="form-field">
            <label for="add-secret">Secret (Base32)</label>
            <input id="add-secret" type="text" class="input input-mono" placeholder="JBSWY3DPEHPK3PXP" required spellcheck="false" autocapitalize="off" />
          </div>

          <div class="form-row">
            <div class="form-field form-field-half">
              <label for="add-type">Type</label>
              <select id="add-type" class="input">
                <option value="totp" selected>TOTP</option>
                <option value="hotp">HOTP</option>
              </select>
            </div>
            <div class="form-field form-field-half">
              <label for="add-algorithm">Algorithm</label>
              <select id="add-algorithm" class="input">
                <option value="SHA1" selected>SHA-1</option>
                <option value="SHA256">SHA-256</option>
                <option value="SHA512">SHA-512</option>
              </select>
            </div>
          </div>

          <div class="form-row">
            <div class="form-field form-field-half">
              <label for="add-digits">Digits</label>
              <select id="add-digits" class="input">
                <option value="6" selected>6</option>
                <option value="8">8</option>
              </select>
            </div>
            <div class="form-field form-field-half">
              <label for="add-period">Period (sec)</label>
              <input id="add-period" type="number" class="input" value="30" min="10" max="120" />
            </div>
          </div>

          <p id="add-error" class="form-error" hidden></p>

          <button id="add-submit" type="submit" class="btn btn-primary btn-full">
            Add Token
          </button>
        </form>
      </main>
    </div>
  `;

  const form = document.getElementById('add-form') as HTMLFormElement;
  const errorEl = document.getElementById('add-error') as HTMLParagraphElement;

  document.getElementById('back-btn')!.addEventListener('click', onDone);

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.hidden = true;

    const issuer = (document.getElementById('add-issuer') as HTMLInputElement).value.trim();
    const account = (document.getElementById('add-account') as HTMLInputElement).value.trim();
    const secret = (document.getElementById('add-secret') as HTMLInputElement).value.trim().replace(/\s+/g, '');
    const tokenType = (document.getElementById('add-type') as HTMLSelectElement).value;
    const algorithm = (document.getElementById('add-algorithm') as HTMLSelectElement).value;
    const digits = parseInt((document.getElementById('add-digits') as HTMLSelectElement).value, 10);
    const period = parseInt((document.getElementById('add-period') as HTMLInputElement).value, 10);

    if (!issuer) {
      showError('Issuer is required.');
      return;
    }
    if (!secret) {
      showError('Secret is required.');
      return;
    }
    // Basic Base32 validation
    if (!/^[A-Z2-7]+=*$/i.test(secret)) {
      showError('Secret must be valid Base32 (A-Z, 2-7).');
      return;
    }

    const btn = document.getElementById('add-submit') as HTMLButtonElement;
    btn.disabled = true;
    btn.textContent = 'Adding…';

    try {
      await tokenAdd({
        issuer,
        account: account || issuer,
        secret: secret.toUpperCase(),
        algorithm,
        digits,
        token_type: tokenType,
        period,
        counter: 0,
        icon: null,
      });
      onDone();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showError(msg);
      btn.disabled = false;
      btn.textContent = 'Add Token';
    }
  });

  function showError(msg: string): void {
    errorEl.textContent = msg;
    errorEl.hidden = false;
  }
}
