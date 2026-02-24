/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */

// Core plugin UI components
export { Meter } from './Meter';
export type { MeterProps } from './Meter';
export { Button } from './Button';
export type { ButtonProps } from './Button';
export { Card } from './Card';
export type { CardProps } from './Card';
export { ErrorMessage } from './ErrorMessage';
export type { ErrorMessageProps } from './ErrorMessage';
export { Switch } from './Switch';
export type { SwitchProps } from './Switch';
export { Knob } from './Knob';
export type { KnobProps } from './Knob';
export { Fader } from './Fader';
export type { FaderProps } from './Fader';
export { Row } from './Row';
export type { RowProps } from './Row';
export { Col } from './Col';
export type { ColProps } from './Col';
export { ParameterSlider } from './ParameterSlider';
export type { ParameterSliderProps } from './ParameterSlider';
export { ParameterGroup } from './ParameterGroup';
export { ParameterToggle } from './ParameterToggle';
export type { ParameterToggleProps } from './ParameterToggle';
export { ParameterSelect } from './ParameterSelect';
export type { ParameterSelectProps } from './ParameterSelect';
export { RadioGroup } from './RadioGroup';
export type { RadioGroupOption, RadioGroupOwnProps as RadioGroupProps } from './RadioGroup';
export { Select } from './Select';
export type { SelectOption, SelectProps } from './Select';
export { VersionBadge } from './VersionBadge';
export { Icon } from './Icon';
export type { IconProps } from './Icon';
export { IconButton } from './IconButton';
export type { IconButtonProps } from './IconButton';

// Connection and status components
export { ConnectionStatus } from './ConnectionStatus';
export type { ConnectionStatusProps } from './ConnectionStatus';
export { LatencyMonitor } from './LatencyMonitor';
export type { LatencyMonitorProps } from './LatencyMonitor';

// Resize components
export { ResizeHandle } from './ResizeHandle';
export type { ResizeHandleProps } from './ResizeHandle';
export { ResizeControls } from './ResizeControls';
export type { ResizeControlsProps } from './ResizeControls';

export type {
  ParameterInfo,
  ParameterType,
  ParameterValue,
  ControlVisualState,
  PluginVisualState,
  MeterFrame,
  AudioRuntimePhase,
  AudioDiagnostic,
  OscilloscopeFrame,
  OscilloscopeChannelView,
  OscilloscopeTriggerMode,
} from './types';

// Audio Processors
export { OscilloscopeProcessor } from './processors/oscilloscope/OscilloscopeProcessor';
export type { OscilloscopeProcessorProps } from './processors/oscilloscope/OscilloscopeProcessor';

export { TestToneProcessor } from './processors/TestToneProcessor';
export type { TestToneProcessorProps } from './processors/TestToneProcessor';

export { GainProcessor } from './processors/GainProcessor';
export type { GainProcessorProps } from './processors/GainProcessor';

export { ToneFilterProcessor } from './processors/ToneFilterProcessor';
export type { ToneFilterProcessorProps } from './processors/ToneFilterProcessor';
