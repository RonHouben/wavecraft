import { useOscilloscopeFrame } from '@wavecraft/core';
import { useHasProcessorInSignalChain } from '@wavecraft/core';
import { Card } from '../../Card';
import { OscilloscopeView } from './OscilloscopeView';
import { useConnectionStatus } from '@wavecraft/core';

export interface OscilloscopeProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OscilloscopeProcessor(
  props: Readonly<OscilloscopeProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscilloscope_tap');
  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return (
    <Card>
      <Card.Header>Oscilloscope</Card.Header>
      <Card.Content>
        <OscilloscopeView connected={connected} frame={frame} />
      </Card.Content>
    </Card>
  );
}
