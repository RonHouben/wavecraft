import React from 'react';
import { useHasProcessorInSignalChain, useAllParametersFor } from '@wavecraft/core';
import { Processor } from './Processor';
import { ProcessorId } from '@wavecraft/core';

export interface ToneFilterProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ToneFilterProcessor(
  props: Readonly<ToneFilterProcessorProps>
): React.JSX.Element | null {
  const id: ProcessorId = 'tone_filter';

  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return <Processor id={id} />;
}
