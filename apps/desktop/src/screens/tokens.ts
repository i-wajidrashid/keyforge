/**
 * Token list screen — the main app view.
 *
 * Displays all tokens with live TOTP codes, countdown timers,
 * copy-to-clipboard on click, and a header with lock / add actions.
 */

import {
  tokenList,
  tokenDelete,
  otpGenerateTotp,
  otpGenerateHotp,
  tokenIncrementCounter,
  vaultLock,
  type Token,
} from '../bridge';
import { renderAddTokenScreen } from './add-token';

/** Format an OTP code with a space in the middle. */
function formatCode(code: string): string {
  if (!code) return '';
  const mid = Math.floor(code.length / 2);
  return `${code.slice(0, mid)} ${code.slice(mid)}`;
}

/** Seconds remaining in the current TOTP period. */
function timeLeft(period: number): number {
  const now = Math.floor(Date.now() / 1000);
  return period - (now % period);
}

// ── Constants ───────────────────────────────────────────────────────

const CODE_PLACEHOLDER = '------';
const URGENCY_DANGER_SECS = 5;
const URGENCY_WARNING_SECS = 10;

function urgencyClass(remaining: number): string {
  if (remaining <= URGENCY_DANGER_SECS) return 'danger';
  if (remaining <= URGENCY_WARNING_SECS) return 'warning';
  return 'safe';
}

// ── State ───────────────────────────────────────────────────────────

let tokens: Token[] = [];
let codes: Map<string, string> = new Map();
let tickTimer: ReturnType<typeof setInterval> | null = null;
let root: HTMLElement;
let onLocked: () => void;

// ── Render ──────────────────────────────────────────────────────────

export function renderTokenList(
  rootEl: HTMLElement,
  onLockedCb: () => void,
): void {
  root = rootEl;
  onLocked = onLockedCb;

  root.innerHTML = `
    <div class="app-shell">
      <header class="app-header">
        <h1 class="app-title">KeyForge</h1>
        <div class="header-actions">
          <button id="add-btn" class="btn-icon" title="Add token" aria-label="Add token">+</button>
          <button id="lock-btn" class="btn-icon" title="Lock vault" aria-label="Lock vault">🔒</button>
        </div>
      </header>
      <main id="token-list" class="token-list">
        <div class="loading">Loading…</div>
      </main>
    </div>
  `;

  document.getElementById('lock-btn')!.addEventListener('click', handleLock);
  document.getElementById('add-btn')!.addEventListener('click', handleAdd);

  loadTokens();
  startTick();
}

async function loadTokens(): Promise<void> {
  try {
    tokens = await tokenList();
    await refreshCodes();
    renderList();
  } catch (err) {
    const list = document.getElementById('token-list')!;
    list.innerHTML = `<div class="empty-state">Failed to load tokens.</div>`;
  }
}

async function refreshCodes(): Promise<void> {
  const newCodes = new Map<string, string>();
  for (const token of tokens) {
    try {
      const code =
        token.token_type === 'hotp'
          ? await otpGenerateHotp(token.id)
          : await otpGenerateTotp(token.id);
      newCodes.set(token.id, code);
    } catch {
      newCodes.set(token.id, CODE_PLACEHOLDER);
    }
  }
  codes = newCodes;
}

function renderList(): void {
  const list = document.getElementById('token-list')!;

  if (tokens.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <p class="empty-title">No tokens yet</p>
        <p class="empty-sub">Tap + to add your first account.</p>
      </div>
    `;
    return;
  }

  list.innerHTML = tokens
    .map((token) => {
      const code = codes.get(token.id) ?? CODE_PLACEHOLDER;
      const remaining = timeLeft(token.period);
      const urgency = urgencyClass(remaining);

      return `
      <div class="token-card" data-id="${token.id}" data-type="${token.token_type}">
        <div class="token-info">
          <span class="token-issuer">${escapeHtml(token.issuer)}</span>
          <span class="token-account">${escapeHtml(token.account)}</span>
        </div>
        <div class="token-code-area">
          <button
            class="token-code"
            data-id="${token.id}"
            title="Click to copy"
            aria-label="Copy code for ${escapeHtml(token.issuer)}"
          >${formatCode(code)}</button>
          ${
            token.token_type === 'totp'
              ? `<div class="token-timer ${urgency}" data-id="${token.id}">${remaining}s</div>`
              : `<button class="token-next-btn" data-id="${token.id}" title="Next code">↻</button>`
          }
        </div>
        <button class="token-delete" data-id="${token.id}" title="Delete" aria-label="Delete ${escapeHtml(token.issuer)}">✕</button>
      </div>
    `;
    })
    .join('');

  // Event delegation
  list.addEventListener('click', handleListClick);
}

// ── Timer ───────────────────────────────────────────────────────────

function startTick(): void {
  if (tickTimer) clearInterval(tickTimer);
  tickTimer = setInterval(async () => {
    // Check if any TOTP period just rolled over
    let needRefresh = false;
    for (const token of tokens) {
      if (token.token_type === 'totp' && timeLeft(token.period) === token.period) {
        needRefresh = true;
        break;
      }
    }

    if (needRefresh) {
      await refreshCodes();
    }

    // Update timers and code displays
    for (const token of tokens) {
      if (token.token_type !== 'totp') continue;

      const remaining = timeLeft(token.period);
      const urgency = urgencyClass(remaining);

      const timerEl = document.querySelector(
        `.token-timer[data-id="${token.id}"]`,
      ) as HTMLElement | null;
      if (timerEl) {
        timerEl.textContent = `${remaining}s`;
        timerEl.className = `token-timer ${urgency}`;
      }

      if (needRefresh) {
        const codeEl = document.querySelector(
          `.token-code[data-id="${token.id}"]`,
        ) as HTMLElement | null;
        if (codeEl) {
          const code = codes.get(token.id) ?? CODE_PLACEHOLDER;
          codeEl.textContent = formatCode(code);
        }
      }
    }
  }, 1000);
}

function stopTick(): void {
  if (tickTimer) {
    clearInterval(tickTimer);
    tickTimer = null;
  }
}

// ── Event handlers ──────────────────────────────────────────────────

async function handleListClick(e: Event): Promise<void> {
  const target = e.target as HTMLElement;
  const id = target.dataset.id;
  if (!id) return;

  // Copy code
  if (target.classList.contains('token-code')) {
    const rawCode = codes.get(id);
    if (rawCode && rawCode !== CODE_PLACEHOLDER) {
      try {
        await navigator.clipboard.writeText(rawCode);
        target.textContent = 'Copied!';
        target.classList.add('copied');
        setTimeout(() => {
          const code = codes.get(id) ?? CODE_PLACEHOLDER;
          target.textContent = formatCode(code);
          target.classList.remove('copied');
        }, 1200);
      } catch {
        /* clipboard unavailable */
      }
    }
    return;
  }

  // HOTP next code
  if (target.classList.contains('token-next-btn')) {
    try {
      await tokenIncrementCounter(id);
      const newCode = await otpGenerateHotp(id);
      codes.set(id, newCode);
      const codeEl = document.querySelector(
        `.token-code[data-id="${id}"]`,
      ) as HTMLElement | null;
      if (codeEl) codeEl.textContent = formatCode(newCode);
    } catch {
      /* ignore */
    }
    return;
  }

  // Delete token
  if (target.classList.contains('token-delete')) {
    const token = tokens.find((t) => t.id === id);
    const name = token ? token.issuer : 'this token';
    if (confirm(`Delete ${name}?`)) {
      try {
        await tokenDelete(id);
        await loadTokens();
      } catch {
        /* ignore */
      }
    }
    return;
  }
}

async function handleLock(): Promise<void> {
  stopTick();
  try {
    await vaultLock();
  } catch {
    /* already locked */
  }
  onLocked();
}

function handleAdd(): void {
  stopTick();
  renderAddTokenScreen(root, () => {
    // After adding (or cancelling), go back to token list
    renderTokenList(root, onLocked);
  });
}

// ── Util ────────────────────────────────────────────────────────────

function escapeHtml(s: string): string {
  const div = document.createElement('div');
  div.textContent = s;
  return div.innerHTML;
}
