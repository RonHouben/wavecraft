import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface OutputGainProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OutputGainProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OutputGainProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="output_gain"
      title="Output Gain"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}
