import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface ExampleProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ExampleProcessor({
  hideWhenNotInSignalChain,
}: Readonly<ExampleProcessorProps>): JSX.Element | null {
  return (
    <SmartProcessor
      id="example_processor"
      title="Example Processor"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
    />
  );
}
