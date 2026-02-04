/**
 * @wavecraft/core/meters - Pure audio math utilities
 *
 * These utilities have no IPC side effects and can be used
 * in tests or standalone applications.
 *
 * @packageDocumentation
 */

/**
 * Convert decibels to linear amplitude
 * @param db Decibels
 */
export declare function dbToLinear(db: number): number;

/**
 * Result from getMeterFrame method
 */
export declare interface GetMeterFrameResult {
    frame: MeterFrame | null;
}

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
export declare function linearToDb(linear: number, floor?: number): number;

/**
 * Metering Types
 *
 * Types related to audio metering.
 */
/**
 * Meter frame data (all values in linear scale, not dB)
 */
export declare interface MeterFrame {
    peak_l: number;
    peak_r: number;
    rms_l: number;
    rms_r: number;
    timestamp: number;
}

export { }
