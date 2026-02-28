/**
 * Toast notification component.
 *
 * Positioned bottom-center per UI-SPEC.md. Auto-dismisses after
 * the specified duration.
 */

const TOAST_DURATION_MS = 2000;
const TOAST_DISMISS_MS = 200;

let container: HTMLElement | null = null;
let dismissTimer: ReturnType<typeof setTimeout> | null = null;

function ensureContainer(): HTMLElement {
  if (!container || !document.body.contains(container)) {
    container = document.createElement('div');
    container.className = 'toast-container';
    document.body.appendChild(container);
  }
  return container;
}

export function showToast(message: string): void {
  const c = ensureContainer();

  // Remove any existing toast after its dismiss animation
  if (dismissTimer) clearTimeout(dismissTimer);
  const existing = c.querySelector('.toast');
  if (existing) {
    existing.classList.add('dismissing');
    setTimeout(() => {
      if (c.contains(existing)) c.removeChild(existing);
    }, TOAST_DISMISS_MS);
  }

  const toast = document.createElement('div');
  toast.className = 'toast';
  toast.textContent = message;
  c.appendChild(toast);

  dismissTimer = setTimeout(() => {
    toast.classList.add('dismissing');
    setTimeout(() => {
      if (c.contains(toast)) c.removeChild(toast);
    }, TOAST_DISMISS_MS);
  }, TOAST_DURATION_MS);
}
