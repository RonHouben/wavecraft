import React from 'react';
import { Processor } from './Processor';

export interface ToneFilterProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ToneFilterProcessor({
  hideWhenNotInSignalChain,
}: Readonly<ToneFilterProcessorProps>): React.JSX.Element | null {
  return <Processor id="tone_filter" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
