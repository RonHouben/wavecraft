/**
 * Oscilloscope types
 */

export type OscilloscopeTriggerMode = 'risingZeroCrossing';

export type OscilloscopeChannelView = 'overlay' | 'left' | 'right';

export interface OscilloscopeFrame {
  points_l: number[];
  points_r: number[];
  sample_rate: number;
  timestamp: number;
  no_signal: boolean;
  trigger_mode: OscilloscopeTriggerMode;
}

export interface GetOscilloscopeFrameResult {
  frame: OscilloscopeFrame | null;
}
