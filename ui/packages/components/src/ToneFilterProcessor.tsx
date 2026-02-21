import React from 'react';
import { useAllParameters, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { parametersForProcessor, processorGroupName } from './processorControlUtils';

export interface ToneFilterProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ToneFilterProcessor(
  props: Readonly<ToneFilterProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('tone_filter');
  const { params } = useAllParameters();

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const groupParameters = parametersForProcessor(params, 'tone_filter');
  if (groupParameters.length === 0) {
    return null;
  }

  return (
    <Processor
      group={{
        name: processorGroupName('tone_filter', 'Tone Filter'),
        parameters: groupParameters,
      }}
    />
  );
}
