/**
 * OscillatorProcessor - Displays oscillator signal status and oscillator-specific controls.
 */

import { useHasProcessorInSignalChain } from '@wavecraft/core';
import React from 'react';
import { Processor } from './Processor';
import { ProcessorId } from '@wavecraft/core';

interface OscillatorProcessorProps {
  hideWhenNotInSignalChain?: boolean;
}

export function OscillatorProcessor(
  props: Readonly<OscillatorProcessorProps>
): React.JSX.Element | null {
  const processorId: ProcessorId = 'oscillator';
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(processorId);

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return <Processor id={processorId} />;
}
