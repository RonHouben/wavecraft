import React from 'react';
import { Processor } from './Processor';

export interface SoftClipProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function SoftClipProcessor({
  hideWhenNotInSignalChain,
}: Readonly<SoftClipProcessorProps>): React.JSX.Element | null {
  return <Processor id="soft_clip" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
