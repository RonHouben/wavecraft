import React from 'react';
import { Processor } from './Processor';

export interface InputTrimProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function InputTrimProcessor({
  hideWhenNotInSignalChain,
}: Readonly<InputTrimProcessorProps>): React.JSX.Element {
  return <Processor id="input_trim" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
