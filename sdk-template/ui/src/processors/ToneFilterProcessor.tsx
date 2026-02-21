import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface ToneFilterProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function ToneFilterProcessor({
  hideWhenNotInSignalChain,
}: Readonly<ToneFilterProcessorProps>): JSX.Element | null {
  return <SmartProcessor id="tone_filter" title="Tone Filter" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
