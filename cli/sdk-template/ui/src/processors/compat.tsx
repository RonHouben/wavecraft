import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

interface LegacyProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

/** @deprecated Use <SmartProcessor id="input_trim" title="Input Trim" .../> directly. */
export function InputTrimProcessor({
  hideWhenNotInSignalChain,
}: Readonly<LegacyProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="input_trim"
      title="Input Trim"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}

/** @deprecated Use <SmartProcessor id="output_gain" title="Output Gain" .../> directly. */
export function OutputGainProcessor({
  hideWhenNotInSignalChain,
}: Readonly<LegacyProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="output_gain"
      title="Output Gain"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}

/** @deprecated Use <SmartProcessor id="oscillator" title="Oscillator" .../> directly. */
export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<LegacyProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="oscillator"
      title="Oscillator"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}

/** @deprecated Use <SmartProcessor id="soft_clip" title="Soft Clip" .../> directly. */
export function SoftClipProcessor({
  hideWhenNotInSignalChain,
}: Readonly<LegacyProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="soft_clip"
      title="Soft Clip"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}

/** @deprecated Use <SmartProcessor id="tone_filter" title="Tone Filter" .../> directly. */
export function ToneFilterProcessor({
  hideWhenNotInSignalChain,
}: Readonly<LegacyProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="tone_filter"
      title="Tone Filter"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}
