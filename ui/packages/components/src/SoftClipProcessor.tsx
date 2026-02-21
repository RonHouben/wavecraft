import React from 'react';
import { useAllParametersFor, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { ProcessorId } from '@wavecraft/core';

export interface SoftClipProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function SoftClipProcessor(
  props: Readonly<SoftClipProcessorProps>
): React.JSX.Element | null {
  const id: ProcessorId = 'soft_clip';

  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return <Processor id={id} />;
}
