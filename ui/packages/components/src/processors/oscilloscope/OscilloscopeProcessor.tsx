import { useConnectionStatus, useOscilloscopeFrame } from '@wavecraft/core';
import { ProcessorCard } from '../ProcessorCard';
import { OscilloscopeView } from './OscilloscopeView';

export interface OscilloscopeProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

export function OscilloscopeProcessor(
  props: Readonly<OscilloscopeProcessorProps>
): React.JSX.Element | null {
  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();

  return (
    <ProcessorCard
      processorId="oscilloscope_tap"
      title="Oscilloscope"
      hideWhenNotInSignalChain={props.hideWhenNotInSignalChain}
      className={props.className}
    >
      <OscilloscopeView connected={connected} frame={frame} />
    </ProcessorCard>
  );
}
