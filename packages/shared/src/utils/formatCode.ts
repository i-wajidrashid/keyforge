/**
 * Format an OTP code with a space in the middle for readability.
 * "123456" → "123 456" (6-digit)
 * "12345678" → "1234 5678" (8-digit)
 */
export function formatCode(code: string): string {
  if (!code || code.length === 0) return '';
  const mid = Math.floor(code.length / 2);
  return `${code.slice(0, mid)} ${code.slice(mid)}`;
}
