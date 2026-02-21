import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface InputTrimProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function InputTrimProcessor({
  hideWhenNotInSignalChain,
}: Readonly<InputTrimProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="input_trim"
      title="Input Trim"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}
