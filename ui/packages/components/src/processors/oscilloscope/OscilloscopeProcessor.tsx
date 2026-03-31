import { type BypassProcessorId, useConnectionStatus, useOscilloscopeFrame } from '@wavecraft/core';
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
    // TODO(Phase 5): Replace ProcessorCard with a TapCard once tap-aware UI is implemented.
    // OscilloscopeTap is a TapProcessor, not a bypass-able Processor — the cast is intentional.
    <ProcessorCard
      processorId={'oscilloscope_tap' as unknown as BypassProcessorId}
      title="Oscilloscope"
      hideWhenNotInSignalChain={props.hideWhenNotInSignalChain}
      className={props.className}
    >
      <OscilloscopeView connected={connected} frame={frame} />
    </ProcessorCard>
  );
}
