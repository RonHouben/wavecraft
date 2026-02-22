import { OscilloscopeProcessor as OscilloscopeView } from '@wavecraft/components';
import {
  useConnectionStatus,
  useHasProcessorInSignalChain,
  useOscilloscopeFrame,
} from '@wavecraft/core';
import type { JSX } from 'react';

export interface OscilloscopeProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OscilloscopeProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscilloscopeProcessorProps>): JSX.Element | null {
  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscilloscope_tap');

  if (hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return <OscilloscopeView connected={connected} frame={frame} />;
}
