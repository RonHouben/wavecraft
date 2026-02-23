import { useOscilloscopeFrame, useConnectionStatus } from '@wavecraft/core';
import { OscilloscopeView } from './OscilloscopeView';
import { ProcessorCard } from '../ProcessorCard';

export interface OscilloscopeProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OscilloscopeProcessor(
  props: Readonly<OscilloscopeProcessorProps>
): React.JSX.Element | null {
  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();

  return (
    <ProcessorCard
      bypassParameterId="oscilloscope_tap_bypass"
      title="Oscilloscope"
      hideWhenNotInSignalChain={props.hideWhenNotInSignalChain}
    >
      <OscilloscopeView connected={connected} frame={frame} />
    </ProcessorCard>
  );
}
