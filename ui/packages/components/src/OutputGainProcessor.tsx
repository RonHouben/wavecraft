import React from 'react';
import { useAllParametersFor, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { ProcessorId } from '@wavecraft/core';

export interface OutputGainProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OutputGainProcessor(
  props: Readonly<OutputGainProcessorProps>
): React.JSX.Element | null {
  const id: ProcessorId = 'output_gain';

  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return <Processor id={id} />;
}
