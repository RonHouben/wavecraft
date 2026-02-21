import React from 'react';
import { useHasProcessorInSignalChain, useAllParametersFor } from '@wavecraft/core';
import { Processor } from '@wavecraft/components';
import { ProcessorId } from '@wavecraft/core';

export interface ExampleProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ExampleProcessor(props: Readonly<ExampleProcessorProps>): React.JSX.Element | null {
  const id: ProcessorId = 'example_processor';

  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return <Processor id={id} />;
}
