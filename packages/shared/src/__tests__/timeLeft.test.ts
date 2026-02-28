import { describe, it, expect } from 'vitest';
import { timeLeft } from '../utils/timeLeft';

describe('timeLeft', () => {
  it('returns full period at exact boundary', () => {
    expect(timeLeft(30, 0)).toBe(30);
    expect(timeLeft(30, 30)).toBe(30);
    expect(timeLeft(30, 60)).toBe(30);
  });

  it('returns correct seconds remaining', () => {
    expect(timeLeft(30, 1)).toBe(29);
    expect(timeLeft(30, 15)).toBe(15);
    expect(timeLeft(30, 29)).toBe(1);
  });

  it('handles 60-second period', () => {
    expect(timeLeft(60, 0)).toBe(60);
    expect(timeLeft(60, 30)).toBe(30);
    expect(timeLeft(60, 59)).toBe(1);
  });
});
