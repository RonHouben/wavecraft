import React from 'react';
import { Processor } from './Processor';

export interface OutputGainProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OutputGainProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OutputGainProcessorProps>): React.JSX.Element | null {
  return <Processor id="output_gain" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
