import { describe, it, expect } from 'vitest';
import { formatCode } from '../utils/formatCode';

describe('formatCode', () => {
  it('formats 6-digit code', () => {
    expect(formatCode('123456')).toBe('123 456');
  });

  it('formats 8-digit code', () => {
    expect(formatCode('12345678')).toBe('1234 5678');
  });

  it('handles empty string', () => {
    expect(formatCode('')).toBe('');
  });

  it('handles single character', () => {
    expect(formatCode('1')).toBe(' 1');
  });

  it('handles two characters', () => {
    expect(formatCode('12')).toBe('1 2');
  });
});
