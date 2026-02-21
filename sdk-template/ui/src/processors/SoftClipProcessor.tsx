import { SmartProcessor } from './SmartProcessor';
import type { JSX } from 'react';

export interface SoftClipProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function SoftClipProcessor({
  hideWhenNotInSignalChain,
}: Readonly<SoftClipProcessorProps>): JSX.Element | null {
  return <SmartProcessor id="soft_clip" title="Soft Clip" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
