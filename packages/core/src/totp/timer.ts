import type { Algorithm } from '@keyforge/shared';
import { generateTOTP } from '../totp/generate';

export interface TotpTickState {
  code: string;
  secondsLeft: number;
}

export type TotpTickCallback = (state: TotpTickState) => void;
export type CodeGenerator = (secret: Uint8Array, time: number, period: number, digits: number, algorithm: Algorithm) => Promise<string>;

/**
 * Creates a TOTP timer that auto-regenerates OTP codes on each period
 * boundary and counts down every second. Returns a `stop` function.
 *
 * The callback fires immediately with the current code, then once per
 * second with updated `secondsLeft`. When the period rolls over, a
 * fresh code is generated and `secondsLeft` resets.
 */
export function startTotpTimer(
  secret: Uint8Array,
  period: number,
  digits: number,
  algorithm: Algorithm,
  onTick: TotpTickCallback,
  generate: CodeGenerator = generateTOTP,
): () => void {
  let timerId: ReturnType<typeof setInterval> | null = null;
  let stopped = false;
  let lastCounter = -1;
  let currentCode = '';

  async function update() {
    if (stopped) return;

    const now = Math.floor(Date.now() / 1000);
    const counter = Math.floor(now / period);
    const secondsLeft = period - (now % period);

    if (counter !== lastCounter) {
      lastCounter = counter;
      currentCode = await generate(secret, now, period, digits, algorithm);
    }

    if (!stopped) {
      onTick({ code: currentCode, secondsLeft });
    }
  }

  // Fire the initial tick, then start the interval
  update().then(() => {
    if (!stopped) {
      timerId = setInterval(update, 1000);
    }
  });

  return () => {
    stopped = true;
    if (timerId !== null) {
      clearInterval(timerId);
      timerId = null;
    }
  };
}
