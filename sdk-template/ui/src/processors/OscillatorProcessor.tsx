import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface OscillatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscillatorProcessorProps>): JSX.Element | null {
  return <SmartProcessor id="oscillator" title="Oscillator" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
