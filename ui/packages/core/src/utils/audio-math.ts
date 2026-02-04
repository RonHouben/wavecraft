/**
 * Audio Math Utilities
 *
 * Pure functions for audio calculations with no side effects.
 */

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
