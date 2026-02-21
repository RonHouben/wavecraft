import React from 'react';
import { useAllParameters, useHasProcessorInSignalChain } from '@wavecraft/core';
import { Processor } from './Processor';
import { parametersForProcessor, processorGroupName } from './processorControlUtils';

export interface InputTrimProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function InputTrimProcessor(
  props: Readonly<InputTrimProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('input_trim');
  const { params } = useAllParameters();

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const groupParameters = parametersForProcessor(params, 'input_trim');
  if (groupParameters.length === 0) {
    return null;
  }

  return (
    <Processor
      group={{
        name: processorGroupName('input_trim', 'Input Trim'),
        parameters: groupParameters,
      }}
    />
  );
}
