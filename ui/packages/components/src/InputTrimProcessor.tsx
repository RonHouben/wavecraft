import React from 'react';
import { useAllParametersFor, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { ProcessorId } from '@wavecraft/core';

export interface InputTrimProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function InputTrimProcessor(
  props: Readonly<InputTrimProcessorProps>
): React.JSX.Element | null {
  const id: ProcessorId = 'input_trim';
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return <Processor id={id} />;
}
