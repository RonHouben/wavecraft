/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */

// Core plugin UI components
export { Meter } from './Meter';
export type { MeterProps } from './Meter';
export { ParameterSlider } from './ParameterSlider';
export type { ParameterSliderProps } from './ParameterSlider';
export { ParameterGroup } from './ParameterGroup';
export { Processor } from './Processor';
export type { ProcessorProps, ProcessorParameter } from './Processor';
export { InputTrimProcessor } from './InputTrimProcessor';
export { OutputGainProcessor } from './OutputGainProcessor';
export { SoftClipProcessor } from './SoftClipProcessor';
export { ToneFilterProcessor } from './ToneFilterProcessor';
export { ParameterToggle } from './ParameterToggle';
export type { ParameterToggleProps } from './ParameterToggle';
export { ParameterSelect } from './ParameterSelect';
export type { ParameterSelectProps } from './ParameterSelect';
export { VersionBadge } from './VersionBadge';

// Connection and status components
export { ConnectionStatus } from './ConnectionStatus';
export type { ConnectionStatusProps } from './ConnectionStatus';
export { LatencyMonitor } from './LatencyMonitor';
export type { LatencyMonitorProps } from './LatencyMonitor';
export { OscillatorProcessor } from './OscillatorProcessor';
export { OscilloscopeProcessor } from './OscilloscopeProcessor';

// Resize components
export { ResizeHandle } from './ResizeHandle';
export type { ResizeHandleProps } from './ResizeHandle';
export { ResizeControls } from './ResizeControls';
export type { ResizeControlsProps } from './ResizeControls';

export type {
  ParameterInfo,
  ParameterType,
  ParameterValue,
  MeterFrame,
  AudioRuntimePhase,
  AudioDiagnostic,
  OscilloscopeFrame,
  OscilloscopeChannelView,
  OscilloscopeTriggerMode,
} from './types';
