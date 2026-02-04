/**
 * Meter polling API for audio visualization
 */

import { IpcBridge } from './IpcBridge';

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
 * Get the latest meter frame from the audio engine
 */
export async function getMeterFrame(): Promise<MeterFrame | null> {
  const bridge = IpcBridge.getInstance();
  const result = await bridge.invoke<GetMeterFrameResult>('getMeterFrame');
  return result.frame;
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
