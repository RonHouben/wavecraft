/**
 * OscillatorProcessor - Displays oscillator signal status and oscillator-specific controls.
 */

import React from 'react';
import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

type OscillatorProcessorProps = Omit<ProcessorProps, 'id'>;

export function OscillatorProcessor({
  title,
  parameters,
}: Readonly<OscillatorProcessorProps>): React.JSX.Element {
  return <Processor id="oscillator" title={title} parameters={parameters} />;
}
