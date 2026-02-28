/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */

// Core plugin UI components
/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */
// Core plugin UI components
export { Button } from './Button';
export type { ButtonProps } from './Button';
export { Card } from './Card';
export type { CardProps } from './Card';
export { Col } from './Col';
export type { ColProps } from './Col';
export { ErrorMessage } from './ErrorMessage';
export type { ErrorMessageProps } from './ErrorMessage';
export { Fader } from './Fader';
export type { FaderProps } from './Fader';
export { Icon } from './Icon';
export type { IconProps } from './Icon';
export { IconButton } from './IconButton';
export type { IconButtonProps } from './IconButton';
export { Knob } from './Knob';
export type { KnobProps } from './Knob';
/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */
// Core plugin UI components
/**
 * @wavecraft/components - Pre-built React UI components for Wavecraft audio plugins.
 *
 * @packageDocumentation
 */
// Core plugin UI components
export { Meter } from './Meter';
export type { MeterProps } from './Meter';
export { ParameterGroup } from './ParameterGroup';
export { ParameterSelect } from './ParameterSelect';
export type { ParameterSelectProps } from './ParameterSelect';
export { ParameterSlider } from './ParameterSlider';
export type { ParameterSliderProps } from './ParameterSlider';
export { ParameterToggle } from './ParameterToggle';
export type { ParameterToggleProps } from './ParameterToggle';
export { RadioGroup } from './RadioGroup';
export type { RadioGroupOption, RadioGroupOwnProps as RadioGroupProps } from './RadioGroup';
export { Row } from './Row';
export type { RowProps } from './Row';
export { Select } from './Select';
export type { SelectOption, SelectProps } from './Select';
export { Switch } from './Switch';
export type { SwitchProps } from './Switch';
export { VersionBadge } from './VersionBadge';

// Connection and status components
// Connection and status components
// Connection and status components
// Connection and status components
export { ConnectionStatus } from './ConnectionStatus';
export type { ConnectionStatusProps } from './ConnectionStatus';
export { LatencyMonitor } from './LatencyMonitor';
export type { LatencyMonitorProps } from './LatencyMonitor';

// Resize components
// Resize components
export { ResizeControls } from './ResizeControls';
export type { ResizeControlsProps } from './ResizeControls';
// Resize components
// Resize components
export { ResizeHandle } from './ResizeHandle';
export type { ResizeHandleProps } from './ResizeHandle';

export type {
  AudioDiagnostic,
  AudioRuntimePhase,
  ControlVisualState,
  MeterFrame,
  OscilloscopeChannelView,
  OscilloscopeFrame,
  OscilloscopeTriggerMode,
  ParameterInfo,
  ParameterType,
  ParameterValue,
  PluginVisualState,
} from './types';

// Audio Processors
// Audio Processors
// Audio Processors
// Audio Processors
export { OscilloscopeProcessor } from './processors/oscilloscope/OscilloscopeProcessor';
export type { OscilloscopeProcessorProps } from './processors/oscilloscope/OscilloscopeProcessor';

export { TestToneProcessor } from './processors/TestToneProcessor';
export type { TestToneProcessorProps } from './processors/TestToneProcessor';

export { GainProcessor } from './processors/GainProcessor';
export type { GainProcessorProps } from './processors/GainProcessor';

export { PassthroughProcessor } from './processors/PassthroughProcessor';
export type { PassthroughProcessorProps } from './processors/PassthroughProcessor';

export { ToneFilterProcessor } from './processors/ToneFilterProcessor';
export type { ToneFilterProcessorProps } from './processors/ToneFilterProcessor';

export { SaturatorProcessor } from './processors/SaturatorProcessor';
export type { SaturatorProcessorProps } from './processors/SaturatorProcessor';

// Signal Chain
// Signal Chain
// Signal Chain
// Signal Chain
export { SignalChain } from './signalChain';
export type { SignalChainProcessorEntry, SignalChainProps } from './signalChain';
