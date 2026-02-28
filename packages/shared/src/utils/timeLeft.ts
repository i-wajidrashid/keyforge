import { MS_PER_SECOND } from '../constants/defaults';

/** Seconds remaining in the current TOTP period. */
export function timeLeft(period: number, now?: number): number {
  const currentTime = now ?? Math.floor(Date.now() / MS_PER_SECOND);
  return period - (currentTime % period);
}
