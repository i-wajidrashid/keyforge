/** Seconds remaining in the current TOTP period. */
export function timeLeft(period: number, now?: number): number {
  const currentTime = now ?? Math.floor(Date.now() / 1000);
  return period - (currentTime % period);
}
