import React from 'react';
import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

export type InputTrimProcessorProps = Omit<ProcessorProps, 'id'>;

export function InputTrimProcessor({
  title,
  parameters,
}: Readonly<InputTrimProcessorProps>): React.JSX.Element {
  return <Processor id="input_trim" title={title} parameters={parameters} />;
}
