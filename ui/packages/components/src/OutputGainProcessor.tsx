import React from 'react';
import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

export type OutputGainProcessorProps = Omit<ProcessorProps, 'id'>;

export function OutputGainProcessor({
  title,
  parameters,
}: Readonly<OutputGainProcessorProps>): React.JSX.Element | null {
  return <Processor id="output_gain" title={title} parameters={parameters} />;
}
