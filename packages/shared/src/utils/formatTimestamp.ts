/**
 * Centralized timestamp formatting utilities.
 *
 * All user-facing timestamps go through these helpers so dates are
 * always localized and consistently formatted across the app.
 */

/**
 * Format a date/time string for display using the user's locale.
 *
 * @param isoOrEpoch - ISO 8601 string, epoch milliseconds, or Date.
 * @param style      - `'short'` (date only), `'medium'` (date + time),
 *                     or `'long'` (full date + time + seconds).
 * @returns A locale-formatted string, or `''` if the input is falsy.
 */
export function formatTimestamp(
  isoOrEpoch: string | number | Date | null | undefined,
  style: 'short' | 'medium' | 'long' = 'medium',
): string {
  if (!isoOrEpoch) return '';

  const date =
    isoOrEpoch instanceof Date ? isoOrEpoch : new Date(isoOrEpoch);

  if (Number.isNaN(date.getTime())) return '';

  const locale = typeof navigator !== 'undefined' ? navigator.language : 'en-US';

  switch (style) {
    case 'short':
      return date.toLocaleDateString(locale, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    case 'long':
      return date.toLocaleString(locale, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
    case 'medium':
    default:
      return date.toLocaleString(locale, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
  }
}

/**
 * Format a relative time description (e.g., "2 minutes ago").
 *
 * Falls back to an absolute timestamp if the Intl.RelativeTimeFormat
 * API is unavailable or the delta is greater than 7 days.
 */
export function formatRelativeTime(
  isoOrEpoch: string | number | Date | null | undefined,
): string {
  if (!isoOrEpoch) return '';

  const date =
    isoOrEpoch instanceof Date ? isoOrEpoch : new Date(isoOrEpoch);

  if (Number.isNaN(date.getTime())) return '';

  const now = Date.now();
  const diffMs = now - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  // Beyond 7 days → fall back to absolute date
  if (diffDay > 7 || typeof Intl?.RelativeTimeFormat === 'undefined') {
    return formatTimestamp(date, 'short');
  }

  const locale = typeof navigator !== 'undefined' ? navigator.language : 'en-US';
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });

  if (diffDay >= 1) return rtf.format(-diffDay, 'day');
  if (diffHour >= 1) return rtf.format(-diffHour, 'hour');
  if (diffMin >= 1) return rtf.format(-diffMin, 'minute');
  return rtf.format(-diffSec, 'second');
}
