/**
 * Add Token screen — manual entry form.
 *
 * Lets the user type in issuer, account, Base32 secret, and optional
 * settings (algorithm, digits, period).  Validates input and calls
 * the Tauri `token_add` command.
 * Follows UI-SPEC.md layout: back arrow icon, proper form fields.
 */

import { tokenAdd } from '../bridge';
import { ICON_BACK } from '../icons';

/** Extract domain from a URL string. */
function extractDomain(url: string): string | null {
  try {
    let normalized = url.trim();
    if (!/^https?:\/\//i.test(normalized)) {
      normalized = `https://${normalized}`;
    }
    const parsed = new URL(normalized);
    return parsed.hostname || null;
  } catch {
    return null;
  }
}

const FAVICON_BASE_URL = 'https://www.google.com/s2/favicons';

export function renderAddTokenScreen(
  root: HTMLElement,
  onDone: () => void,
): void {
  root.innerHTML = `
    <div class="app-shell">
      <header class="app-header">
        <button id="back-btn" class="btn-icon" title="Back" aria-label="Back">${ICON_BACK}</button>
        <h1 class="app-title">Add Token</h1>
        <div class="header-actions"></div>
      </header>
      <main class="add-form-container">
        <form id="add-form" class="add-form" autocomplete="off">
          <div class="form-field">
            <label for="add-issuer">Issuer (service name)</label>
            <input id="add-issuer" type="text" class="input" placeholder="GitHub" required />
          </div>

          <div class="form-field">
            <label for="add-account">Account</label>
            <input id="add-account" type="text" class="input" placeholder="user@email.com" />
          </div>

          <div class="form-field">
            <label for="add-secret">Secret Key</label>
            <input id="add-secret" type="text" class="input input-mono" placeholder="JBSWY3DPEHPK3PXP" required spellcheck="false" autocapitalize="off" />
          </div>

          <div class="form-field">
            <label for="add-website">Website URL (optional)</label>
            <input id="add-website" type="text" class="input" placeholder="github.com" spellcheck="false" />
          </div>

          <div class="form-field">
              <label for="add-type">Type</label>
              <select id="add-type" class="input">
                <option value="totp" selected>TOTP</option>
                <option value="hotp">HOTP</option>
              </select>
          </div>

          <button id="advanced-toggle" type="button" class="advanced-toggle">
            Advanced Options
          </button>

          <div id="advanced-section" class="advanced-section" hidden>
            <div class="form-row">
              <div class="form-field form-field-half">
                <label for="add-algorithm">Algorithm</label>
                <select id="add-algorithm" class="input">
                  <option value="SHA1" selected>SHA-1</option>
                  <option value="SHA256">SHA-256</option>
                  <option value="SHA512">SHA-512</option>
                </select>
              </div>
              <div class="form-field form-field-half">
                <label for="add-digits">Digits</label>
                <select id="add-digits" class="input">
                  <option value="6" selected>6</option>
                  <option value="8">8</option>
                </select>
              </div>
            </div>

            <div class="form-field">
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

  // Advanced toggle
  const advancedToggle = document.getElementById('advanced-toggle')!;
  const advancedSection = document.getElementById('advanced-section')!;
  advancedToggle.addEventListener('click', () => {
    const isHidden = advancedSection.hidden;
    advancedSection.hidden = !isHidden;
    advancedToggle.classList.toggle('open', isHidden);
  });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.hidden = true;

    const issuer = (document.getElementById('add-issuer') as HTMLInputElement).value.trim();
    const account = (document.getElementById('add-account') as HTMLInputElement).value.trim();
    const secret = (document.getElementById('add-secret') as HTMLInputElement).value.trim().replace(/\s+/g, '');
    const website = (document.getElementById('add-website') as HTMLInputElement).value.trim();
    const tokenType = (document.getElementById('add-type') as HTMLSelectElement).value;
    const algorithm = (document.getElementById('add-algorithm') as HTMLSelectElement).value;
    const digits = parseInt((document.getElementById('add-digits') as HTMLSelectElement).value, 10);
    const period = parseInt((document.getElementById('add-period') as HTMLInputElement).value, 10);

    if (!issuer) {
      showError('Issuer is required');
      return;
    }
    if (!secret) {
      showError('Secret key is required');
      return;
    }
    if (!/^[A-Z2-7]+=*$/i.test(secret)) {
      showError('Invalid Base32 secret');
      return;
    }

    // Build favicon URL from website domain
    let icon: string | null = null;
    if (website) {
      const domain = extractDomain(website);
      if (domain) {
        icon = `${FAVICON_BASE_URL}?domain=${encodeURIComponent(domain)}&sz=64`;
      }
    }

    const btn = document.getElementById('add-submit') as HTMLButtonElement;
    btn.disabled = true;
    btn.textContent = 'Adding\u2026';

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
        icon,
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
