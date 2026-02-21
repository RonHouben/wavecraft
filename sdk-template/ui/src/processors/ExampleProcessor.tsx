import React from 'react';
import { useHasProcessorInSignalChain, useAllParametersFor } from '@wavecraft/core';
import { Processor } from '@wavecraft/components';

export interface ExampleProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ExampleProcessor(props: Readonly<ExampleProcessorProps>): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('example_processor');
  const { params } = useAllParametersFor('example_processor');

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return (
    <Processor
      group={{
        name: 'Example Processor',
        parameters: params,
      }}
    />
  );
}
