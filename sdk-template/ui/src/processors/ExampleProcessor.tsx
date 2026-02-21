import React from 'react';
import { Processor } from '@wavecraft/components';

export interface ExampleProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ExampleProcessor({
  hideWhenNotInSignalChain,
}: Readonly<ExampleProcessorProps>): React.JSX.Element | null {
  return <Processor id="example_processor" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
