import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { startTotpTimer } from '../totp/timer';

describe('startTotpTimer', () => {
  // Synchronous mock generator — avoids crypto.subtle issues with fake timers
  let callCount = 0;
  const mockGenerate = vi.fn(async () => {
    callCount++;
    return String(callCount).padStart(6, '0');
  });

  beforeEach(() => {
    vi.useFakeTimers();
    callCount = 0;
    mockGenerate.mockClear();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const secret = new Uint8Array(20);

  it('calls back immediately with a code and secondsLeft', async () => {
    const cb = vi.fn();
    vi.setSystemTime(new Date(30_000)); // t=30s (start of period)

    const stop = startTotpTimer(secret, 30, 6, 'SHA1', cb, mockGenerate);

    // Flush microtasks from the initial async update
    await vi.advanceTimersByTimeAsync(0);

    expect(cb).toHaveBeenCalled();
    const state = cb.mock.calls[0][0];
    expect(state.code).toHaveLength(6);
    expect(state.secondsLeft).toBe(30);

    stop();
  });

  it('ticks every second with updated secondsLeft', async () => {
    const cb = vi.fn();
    vi.setSystemTime(new Date(30_000));

    const stop = startTotpTimer(secret, 30, 6, 'SHA1', cb, mockGenerate);

    await vi.advanceTimersByTimeAsync(0);    // initial
    await vi.advanceTimersByTimeAsync(1000); // +1s
    await vi.advanceTimersByTimeAsync(1000); // +2s

    expect(cb.mock.calls.length).toBeGreaterThanOrEqual(3);
    // secondsLeft should decrease
    expect(cb.mock.calls[1][0].secondsLeft).toBe(29);
    expect(cb.mock.calls[2][0].secondsLeft).toBe(28);

    stop();
  });

  it('regenerates code on period boundary', async () => {
    const cb = vi.fn();

    // Start 2 seconds before boundary (t=28)
    vi.setSystemTime(new Date(28_000));

    const stop = startTotpTimer(secret, 30, 6, 'SHA1', cb, mockGenerate);

    await vi.advanceTimersByTimeAsync(0);    // t=28 → generate called (counter=0)
    await vi.advanceTimersByTimeAsync(1000); // t=29, same counter
    await vi.advanceTimersByTimeAsync(1000); // t=30 → new counter=1, generate called again

    // mockGenerate should have been called twice (once per counter)
    expect(mockGenerate).toHaveBeenCalledTimes(2);

    stop();
  });

  it('stops ticking after stop() is called', async () => {
    const cb = vi.fn();
    vi.setSystemTime(new Date(30_000));

    const stop = startTotpTimer(secret, 30, 6, 'SHA1', cb, mockGenerate);

    await vi.advanceTimersByTimeAsync(0);
    stop();

    const callsBefore = cb.mock.calls.length;
    await vi.advanceTimersByTimeAsync(5000);
    expect(cb.mock.calls.length).toBe(callsBefore);
  });
});
