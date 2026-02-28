/**
 * Token list screen — the main app view.
 *
 * Displays all tokens with live TOTP codes, SVG progress rings,
 * copy-to-clipboard via Tauri plugin, search, and header actions.
 * Follows UI-SPEC.md layout and interactions.
 */

import {
  tokenList,
  tokenDelete,
  otpGenerateTotp,
  otpGenerateHotp,
  tokenIncrementCounter,
  vaultLock,
  clipboardWrite,
  clipboardRead,
  type Token,
} from '../bridge';
import { renderAddTokenScreen } from './add-token';
import { showToast } from '../toast';
import {
  ICON_PLUS,
  ICON_LOCK,
  ICON_SEARCH_POSITIONED,
  ICON_REFRESH,
  ICON_X,
  ICON_TRASH,
} from '../icons';

// ── Constants (from @keyforge/shared where applicable) ──────────────

const MS_PER_SECOND = 1000;
const TIMER_INTERVAL_MS = 1000;
const URGENCY_DANGER_SECS = 5;
const URGENCY_WARNING_SECS = 10;
const PROGRESS_RING_RADIUS = 8;
const PROGRESS_RING_CIRCUMFERENCE = 2 * Math.PI * PROGRESS_RING_RADIUS;
const FAVICON_URL_PREFIX = 'https://www.google.com/s2/favicons';

// ── Helpers ─────────────────────────────────────────────────────────

/** Format an OTP code with a space in the middle per UI-SPEC.md. */
function formatCode(code: string): string {
  if (!code) return '';
  const mid = Math.floor(code.length / 2);
  return `${code.slice(0, mid)}\u2009${code.slice(mid)}`;
}

/** Seconds remaining in the current TOTP period. */
function timeLeft(period: number): number {
  const now = Math.floor(Date.now() / MS_PER_SECOND);
  return period - (now % period);
}

/** SVG stroke color based on time remaining. */
function urgencyColor(remaining: number): string {
  if (remaining <= URGENCY_DANGER_SECS) return 'var(--color-accent-danger)';
  if (remaining <= URGENCY_WARNING_SECS) return 'var(--color-accent-warning)';
  return 'var(--color-accent-safe)';
}

/** Compute stroke-dashoffset for the progress ring. */
function ringOffset(remaining: number, period: number): number {
  const progress = remaining / period;
  return PROGRESS_RING_CIRCUMFERENCE * (1 - progress);
}

/** Render a progress ring SVG — UI-SPEC.md: 20px, 2.5px stroke. */
function progressRingSvg(remaining: number, period: number): string {
  const offset = ringOffset(remaining, period);
  const color = urgencyColor(remaining);
  return `<svg class="progress-ring" viewBox="0 0 20 20">
    <circle class="progress-ring__bg" cx="10" cy="10" r="${PROGRESS_RING_RADIUS}"/>
    <circle class="progress-ring__fg" cx="10" cy="10" r="${PROGRESS_RING_RADIUS}"
      stroke="${color}"
      stroke-dasharray="${PROGRESS_RING_CIRCUMFERENCE}"
      stroke-dashoffset="${offset}"/>
  </svg>`;
}

/** Cached div for HTML escaping — reused to avoid allocating a new element per call. */
const _escDiv = document.createElement('div');
function escapeHtml(s: string): string {
  _escDiv.textContent = s;
  return _escDiv.innerHTML;
}

// ── State ───────────────────────────────────────────────────────────

let tokens: Token[] = [];
let codes: Map<string, string> = new Map();
let tickTimer: ReturnType<typeof setInterval> | null = null;
let refreshing = false; // guard against overlapping async refreshCodes calls
let searchQuery = '';
let root: HTMLElement;
let onLocked: () => void;
let clipboardClearTimer: ReturnType<typeof setTimeout> | null = null;
let lastCopiedCode: string | null = null;

// ── Render ──────────────────────────────────────────────────────────

export function renderTokenList(
  rootEl: HTMLElement,
  onLockedCb: () => void,
): void {
  // Clean up any previous listeners before re-rendering
  cleanup();

  root = rootEl;
  onLocked = onLockedCb;
  searchQuery = '';

  root.innerHTML = `
    <div class="app-shell">
      <header class="app-header">
        <div class="search-wrapper">
          ${ICON_SEARCH_POSITIONED}
          <input id="search-input" class="search-input" type="text"
            placeholder="Search" spellcheck="false" autocomplete="off" />
          <button id="search-clear" class="search-clear" type="button" aria-label="Clear search">${ICON_X}</button>
        </div>
        <div class="header-actions">
          <button id="add-btn" class="btn-icon" title="Add token" aria-label="Add token">${ICON_PLUS}</button>
          <button id="lock-btn" class="btn-icon" title="Lock vault" aria-label="Lock vault">${ICON_LOCK}</button>
        </div>
      </header>
      <main id="token-list" class="token-list" role="list" aria-live="polite">
        <div class="loading">Loading tokens…</div>
      </main>
    </div>
  `;

  document.getElementById('lock-btn')!.addEventListener('click', handleLock);
  document.getElementById('add-btn')!.addEventListener('click', handleAdd);

  const searchInput = document.getElementById('search-input') as HTMLInputElement;
  searchInput.addEventListener('input', () => {
    searchQuery = searchInput.value.toLowerCase();
    filterList();
  });
  document.getElementById('search-clear')!.addEventListener('click', () => {
    searchInput.value = '';
    searchQuery = '';
    filterList();
    searchInput.focus();
  });

  // Keyboard shortcuts (UI-SPEC.md)
  document.addEventListener('keydown', handleKeyboard);

  loadTokens();
  startTick();
}

async function loadTokens(): Promise<void> {
  try {
    tokens = await tokenList();
    await refreshCodes();
    renderList();
  } catch {
    const list = document.getElementById('token-list');
    if (list) list.innerHTML = `<div class="empty-state"><p class="empty-title">Failed to load tokens</p></div>`;
  }
}

async function refreshCodes(): Promise<void> {
  if (refreshing) return; // prevent overlapping calls
  refreshing = true;
  try {
    const newCodes = new Map<string, string>();
    for (const token of tokens) {
      try {
        const code =
          token.token_type === 'hotp'
            ? await otpGenerateHotp(token.id)
            : await otpGenerateTotp(token.id);
        newCodes.set(token.id, code);
      } catch {
        // Leave empty — no placeholder strings
      }
    }
    codes = newCodes;
  } finally {
    refreshing = false;
  }
}

function renderList(): void {
  const list = document.getElementById('token-list');
  if (!list) return;

  if (tokens.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <p class="empty-title">No tokens yet</p>
        <p class="empty-sub">Add your first token to get started.</p>
        <button id="empty-add-btn" class="empty-action">Add your first token</button>
      </div>
    `;
    document.getElementById('empty-add-btn')?.addEventListener('click', handleAdd);
    return;
  }

  list.innerHTML = tokens
    .map((token) => {
      const code = codes.get(token.id) ?? '';
      const remaining = timeLeft(token.period);
      const initial = (token.issuer || '?')[0].toUpperCase();
      const avatarHtml = token.icon && token.icon.startsWith(FAVICON_URL_PREFIX)
        ? `<img class="token-avatar token-avatar-img" src="${escapeHtml(token.icon)}" alt="" data-initial="${initial}" /><div class="token-avatar token-avatar-letter" style="display:none">${initial}</div>`
        : `<div class="token-avatar">${initial}</div>`;

      return `
      <div class="token-card" role="listitem" data-id="${token.id}" data-type="${token.token_type}"
           data-issuer="${escapeHtml(token.issuer.toLowerCase())}"
           data-account="${escapeHtml(token.account.toLowerCase())}">
        ${avatarHtml}
        <div class="token-info">
          <span class="token-issuer">${escapeHtml(token.issuer)}</span>
          <span class="token-account">${escapeHtml(token.account)}</span>
        </div>
        <div class="token-right">
          <span class="token-code" data-id="${token.id}">${formatCode(code)}</span>
          <div class="token-timer-row">
            ${
              token.token_type === 'totp'
                ? `${progressRingSvg(remaining, token.period)}<span class="token-timer-text" data-id="${token.id}">${remaining}s</span>`
                : `<button class="token-hotp-refresh" data-id="${token.id}" title="Next code" aria-label="Next code">${ICON_REFRESH}</button>`
            }
          </div>
        </div>
        <button class="token-delete-btn" data-id="${token.id}" title="Delete token" aria-label="Delete token">${ICON_TRASH}</button>
      </div>
    `;
    })
    .join('');

  // Handle favicon load errors — fall back to letter avatar
  list.querySelectorAll('.token-avatar-img').forEach((img) => {
    img.addEventListener('error', () => {
      (img as HTMLElement).style.display = 'none';
      const fallback = img.nextElementSibling as HTMLElement | null;
      if (fallback) fallback.style.display = 'flex';
    });
  });

  // Single event delegation for the entire list
  list.onclick = handleListClick;

  filterList();
}

function filterList(): void {
  const cards = document.querySelectorAll('.token-card') as NodeListOf<HTMLElement>;
  for (const card of cards) {
    if (!searchQuery) {
      card.classList.remove('hidden');
      continue;
    }
    const issuer = card.dataset.issuer ?? '';
    const account = card.dataset.account ?? '';
    const matches = issuer.includes(searchQuery) || account.includes(searchQuery);
    card.classList.toggle('hidden', !matches);
  }
}

// ── Timer ───────────────────────────────────────────────────────────

function startTick(): void {
  if (tickTimer) clearInterval(tickTimer);
  tickTimer = setInterval(async () => {
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

    for (const token of tokens) {
      if (token.token_type !== 'totp') continue;

      const remaining = timeLeft(token.period);
      const color = urgencyColor(remaining);
      const offset = ringOffset(remaining, token.period);

      // Update progress ring
      const card = document.querySelector(`.token-card[data-id="${token.id}"]`);
      if (!card) continue;

      const ringFg = card.querySelector('.progress-ring__fg') as SVGCircleElement | null;
      if (ringFg) {
        ringFg.style.strokeDashoffset = String(offset);
        ringFg.setAttribute('stroke', color);
      }

      // Update timer text
      const timerText = card.querySelector('.token-timer-text') as HTMLElement | null;
      if (timerText) {
        timerText.textContent = `${remaining}s`;
      }

      // Update code on refresh with fade animation
      if (needRefresh) {
        const codeEl = card.querySelector('.token-code') as HTMLElement | null;
        if (codeEl) {
          const code = codes.get(token.id) ?? '';
          codeEl.textContent = formatCode(code);
          codeEl.classList.add('refreshing');
          setTimeout(() => codeEl.classList.remove('refreshing'), 150);
        }
      }
    }
  }, TIMER_INTERVAL_MS);
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

  // Delete button
  const deleteBtn = target.closest('.token-delete-btn') as HTMLElement | null;
  if (deleteBtn) {
    e.stopPropagation();
    const id = deleteBtn.dataset.id;
    if (!id) return;
    const token = tokens.find((t) => t.id === id);
    if (!token) return;
    if (!confirm(`Delete ${token.issuer}?`)) return;
    try {
      await tokenDelete(id);
      await loadTokens();
      showToast('Token deleted');
    } catch { /* ignore */ }
    return;
  }

  // HOTP refresh button
  const refreshBtn = target.closest('.token-hotp-refresh') as HTMLElement | null;
  if (refreshBtn) {
    const id = refreshBtn.dataset.id;
    if (!id) return;
    try {
      await tokenIncrementCounter(id);
      const newCode = await otpGenerateHotp(id);
      codes.set(id, newCode);
      const codeEl = document.querySelector(`.token-code[data-id="${id}"]`) as HTMLElement | null;
      if (codeEl) codeEl.textContent = formatCode(newCode);
    } catch { /* ignore */ }
    return;
  }

  // Copy code on card click — UI-SPEC.md: tap/click copies code, flash card, toast
  const card = target.closest('.token-card') as HTMLElement | null;
  if (card) {
    const id = card.dataset.id;
    if (!id) return;
    const rawCode = codes.get(id);
    if (!rawCode) return;

    try {
      await clipboardWrite(rawCode);
      card.classList.add('flash');
      setTimeout(() => card.classList.remove('flash'), 200);
      showToast('Copied');

      // Schedule clipboard auto-clear when the current TOTP period expires
      const token = tokens.find((t) => t.id === id);
      if (token && token.token_type === 'totp') {
        if (clipboardClearTimer) clearTimeout(clipboardClearTimer);
        lastCopiedCode = rawCode;
        const remaining = timeLeft(token.period);
        clipboardClearTimer = setTimeout(async () => {
          try {
            const current = await clipboardRead();
            if (current === lastCopiedCode) {
              await clipboardWrite('');
            }
          } catch { /* clipboard unavailable */ }
          lastCopiedCode = null;
          clipboardClearTimer = null;
        }, remaining * MS_PER_SECOND);
      }
    } catch { /* clipboard unavailable */ }
    return;
  }
}

async function handleLock(): Promise<void> {
  cleanup();
  try {
    await vaultLock();
  } catch { /* already locked */ }
  onLocked();
}

function handleAdd(): void {
  cleanup();
  renderAddTokenScreen(root, () => {
    renderTokenList(root, onLocked);
  });
}

function handleKeyboard(e: KeyboardEvent): void {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key === 'f') {
    e.preventDefault();
    document.getElementById('search-input')?.focus();
  } else if (mod && e.key === 'n') {
    e.preventDefault();
    handleAdd();
  } else if (mod && e.key === 'l') {
    e.preventDefault();
    handleLock();
  } else if (e.key === 'Escape') {
    const searchInput = document.getElementById('search-input') as HTMLInputElement | null;
    if (searchInput && searchInput.value) {
      searchInput.value = '';
      searchQuery = '';
      filterList();
    }
  }
}

function cleanup(): void {
  stopTick();
  if (clipboardClearTimer) {
    clearTimeout(clipboardClearTimer);
    clipboardClearTimer = null;
  }
  lastCopiedCode = null;
  document.removeEventListener('keydown', handleKeyboard);
  tokens = [];
  codes = new Map();
  refreshing = false;
  searchQuery = '';
}
