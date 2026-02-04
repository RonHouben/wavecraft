/**
 * Metering Types
 *
 * Types related to audio metering.
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
