import type { ElementType, ComponentPropsWithoutRef } from 'react';

export type ParameterType = 'float' | 'bool' | 'enum';
export type ParameterValue = number | boolean | string;
export type ParameterVariant = string | undefined;

export type ControlVisualState =
  | 'default'
  | 'hover'
  | 'focus'
  | 'active'
  | 'disabled'
  | 'loading'
  | 'error';
export type PluginVisualState = 'bypassed' | 'armed' | 'mapped';

export interface ParameterInfo<
  T extends ParameterValue = ParameterValue,
  V extends ParameterVariant = ParameterVariant,
> {
  id: string;
  name: string;
  type: ParameterType;
  value: T;
  default: T;
  min: number;
  max: number;
  unit?: string;
  group?: string;
  variants?: V[];
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

export type PolymorphicProps<C extends ElementType, OwnProps> = OwnProps & {
  as?: C;
} & Omit<ComponentPropsWithoutRef<C>, keyof OwnProps | 'as'>;
