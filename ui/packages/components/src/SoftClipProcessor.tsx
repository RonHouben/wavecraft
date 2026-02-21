import React from 'react';
import { useAllParameters, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { parametersForProcessor, processorGroupName } from './processorControlUtils';

export interface SoftClipProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function SoftClipProcessor(
  props: Readonly<SoftClipProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('soft_clip');
  const { params } = useAllParameters();

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const groupParameters = parametersForProcessor(params, 'soft_clip');
  if (groupParameters.length === 0) {
    return null;
  }

  return (
    <Processor
      group={{
        name: processorGroupName('soft_clip', 'Soft Clip'),
        parameters: groupParameters,
      }}
    />
  );
}
