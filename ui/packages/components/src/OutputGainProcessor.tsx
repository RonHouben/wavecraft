import React from 'react';
import { useAllParameters, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { parametersForProcessor, processorGroupName } from './processorControlUtils';

export interface OutputGainProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OutputGainProcessor(
  props: Readonly<OutputGainProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('output_gain');
  const { params } = useAllParameters();

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const groupParameters = parametersForProcessor(params, 'output_gain');
  if (groupParameters.length === 0) {
    return null;
  }

  return (
    <Processor
      group={{
        name: processorGroupName('output_gain', 'Output Gain'),
        parameters: groupParameters,
      }}
    />
  );
}
