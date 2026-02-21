import React from 'react';
import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

export type SoftClipProcessorProps = Omit<ProcessorProps, 'id'>;

export function SoftClipProcessor({
  title,
  parameters,
}: Readonly<SoftClipProcessorProps>): React.JSX.Element | null {
  return <Processor id="soft_clip" title={title} parameters={parameters} />;
}
