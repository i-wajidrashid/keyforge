import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { copyToClipboard, clearClipboard, cancelScheduledClear } from '../utils/clipboard';

describe('clipboard', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    // Mock navigator.clipboard
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockResolvedValue(undefined),
        readText: vi.fn().mockResolvedValue(''),
      },
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    cancelScheduledClear();
    vi.useRealTimers();
  });

  it('copies text to clipboard', async () => {
    const result = await copyToClipboard('123456');
    expect(result).toBe(true);
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('123456');
  });

  it('returns false when clipboard write fails', async () => {
    vi.mocked(navigator.clipboard.writeText).mockRejectedValueOnce(new Error('denied'));
    const result = await copyToClipboard('123456');
    expect(result).toBe(false);
  });

  it('auto-clears clipboard after default timeout', async () => {
    await copyToClipboard('123456');
    expect(navigator.clipboard.writeText).toHaveBeenCalledTimes(1);

    // Advance 30s (default clear delay)
    await vi.advanceTimersByTimeAsync(30_000);
    // writeText should have been called again with empty string
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('');
  });

  it('auto-clears clipboard after custom timeout', async () => {
    await copyToClipboard('123456', 5000);
    expect(navigator.clipboard.writeText).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(5000);
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('');
  });

  it('cancels previous clear timer on new copy', async () => {
    await copyToClipboard('111111', 10_000);
    await copyToClipboard('222222', 10_000);

    // After 10s, only the second timer should fire
    await vi.advanceTimersByTimeAsync(10_000);
    // Should be: '111111', '222222', '' (clear)
    expect(navigator.clipboard.writeText).toHaveBeenCalledTimes(3);
    expect(navigator.clipboard.writeText).toHaveBeenLastCalledWith('');
  });

  it('clearClipboard writes empty string', async () => {
    await clearClipboard();
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('');
  });
});
