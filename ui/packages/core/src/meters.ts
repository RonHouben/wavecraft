/**
 * @wavecraft/core/meters - Pure audio math utilities
 *
 * These utilities have no IPC side effects and can be used
 * in tests or standalone applications.
 *
 * @packageDocumentation
 */

/**
 * Meter frame data (all values in linear scale, not dB)
 */
export interface MeterFrame {
  peak_l: number;
  peak_r: number;
  rms_l: number;
  rms_r: number;
  timestamp: number;
}

/**
 * Result from getMeterFrame method
 */
export interface GetMeterFrameResult {
  frame: MeterFrame | null;
}

/**
 * Convert linear amplitude to decibels
 * @param linear Linear amplitude (0.0 to 1.0+)
 * @param floor Minimum dB value to return (default: -60)
 */
export function linearToDb(linear: number, floor: number = -60): number {
  if (linear <= 0) {
    return floor;
  }
  const db = 20 * Math.log10(linear);
  return Math.max(db, floor);
}

/**
 * Convert decibels to linear amplitude
 * @param db Decibels
 */
export function dbToLinear(db: number): number {
  return Math.pow(10, db / 20);
}
