import React from 'react';
import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

export type ToneFilterProcessorProps = Omit<ProcessorProps, 'id'>;

export function ToneFilterProcessor({
  title,
  parameters,
}: Readonly<ToneFilterProcessorProps>): React.JSX.Element | null {
  return <Processor id="tone_filter" title={title} parameters={parameters} />;
}
