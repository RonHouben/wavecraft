/**
 * @wavecraft/core/meters - Pure audio math utilities
 *
 * These utilities have no IPC side effects and can be used
 * in tests or standalone applications.
 *
 * @packageDocumentation
 */

// Re-export types from canonical location
export type { MeterFrame, GetMeterFrameResult } from './types/metering';

// Re-export audio math utilities
export { linearToDb, dbToLinear } from './utils/audio-math';
