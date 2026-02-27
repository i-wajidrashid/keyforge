import { describe, it, expect } from 'vitest';
import { formatTimestamp, formatRelativeTime } from '../utils/formatTimestamp';

describe('formatTimestamp', () => {
  it('returns empty string for null/undefined', () => {
    expect(formatTimestamp(null)).toBe('');
    expect(formatTimestamp(undefined)).toBe('');
    expect(formatTimestamp('')).toBe('');
  });

  it('returns empty string for invalid date input', () => {
    expect(formatTimestamp('not-a-date')).toBe('');
  });

  it('formats an ISO string with default (medium) style', () => {
    const result = formatTimestamp('2025-06-15T10:30:00Z');
    // Should produce a locale-dependent string with date + time (no seconds)
    expect(result.length).toBeGreaterThan(0);
    expect(result).toContain('2025');
  });

  it('formats with short style (date only)', () => {
    const result = formatTimestamp('2025-06-15T10:30:00Z', 'short');
    expect(result.length).toBeGreaterThan(0);
    expect(result).toContain('2025');
  });

  it('formats with long style (date + time + seconds)', () => {
    const result = formatTimestamp('2025-06-15T10:30:45Z', 'long');
    expect(result.length).toBeGreaterThan(0);
    expect(result).toContain('2025');
  });

  it('accepts epoch milliseconds', () => {
    const result = formatTimestamp(1718444400000); // 2024-06-15
    expect(result.length).toBeGreaterThan(0);
    expect(result).toContain('2024');
  });

  it('accepts a Date object', () => {
    const result = formatTimestamp(new Date('2025-01-01'));
    expect(result.length).toBeGreaterThan(0);
    expect(result).toContain('2025');
  });
});

describe('formatRelativeTime', () => {
  it('returns empty string for null/undefined', () => {
    expect(formatRelativeTime(null)).toBe('');
    expect(formatRelativeTime(undefined)).toBe('');
  });

  it('returns empty string for invalid date input', () => {
    expect(formatRelativeTime('not-a-date')).toBe('');
  });

  it('returns a non-empty string for a recent timestamp', () => {
    const recent = new Date(Date.now() - 30_000); // 30 seconds ago
    const result = formatRelativeTime(recent);
    expect(result.length).toBeGreaterThan(0);
  });

  it('returns a non-empty string for an old timestamp', () => {
    const old = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000); // 30 days ago
    const result = formatRelativeTime(old);
    expect(result.length).toBeGreaterThan(0);
  });
});
