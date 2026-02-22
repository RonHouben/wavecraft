export type ParameterType = 'float' | 'bool' | 'enum';
export type ParameterValue = number | boolean;

export type ControlVisualState = 'default' | 'loading' | 'error';
export type PluginVisualState = 'bypassed' | 'armed' | 'mapped';

export interface ParameterInfo {
  id: string;
  name: string;
  type: ParameterType;
  value: ParameterValue;
  default: ParameterValue;
  min: number;
  max: number;
  unit?: string;
  group?: string;
  variants?: string[];
}

export interface MeterFrame {
  peak_l: number;
  peak_r: number;
  rms_l: number;
  rms_r: number;
  timestamp: number;
}

export type AudioRuntimePhase =
  | 'disabled'
  | 'initializing'
  | 'runningFullDuplex'
  | 'runningInputOnly'
  | 'degraded'
  | 'failed';

export interface AudioDiagnostic {
  code: string;
  message: string;
  hint?: string;
}

export interface OscilloscopeFrame {
  points_l: number[];
  points_r: number[];
  sample_rate: number;
  timestamp: number;
  no_signal: boolean;
  trigger_mode: string;
}

export type OscilloscopeChannelView = 'overlay' | 'left' | 'right';
export type OscilloscopeTriggerMode = 'risingZeroCrossing';
