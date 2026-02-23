import { useOscilloscopeFrame } from '@wavecraft/core';
import { useHasProcessorInSignalChain } from '@wavecraft/core';
import { Card } from '../../Card';
import { OscilloscopeView } from './OscilloscopeView';
import { useConnectionStatus } from '@wavecraft/core';
import { useParametersForProcessor } from '@wavecraft/core';
import { useParameter } from '@wavecraft/core';
import { Switch } from '../../Switch';

export interface OscilloscopeProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function OscilloscopeProcessor(
  props: Readonly<OscilloscopeProcessorProps>
): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscilloscope_tap');

  const { connected } = useConnectionStatus();
  const frame = useOscilloscopeFrame();

  const { param: bypassParam, setValue: setBypassValue } = useParameter('oscilloscope_tap_bypass');

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return (
    <Card
      data-bypassed={bypassParam?.value}
      className={`w-fit rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel transition-[opacity,filter] duration-150 ${
        bypassParam?.value ? 'opacity-70 brightness-90 saturate-50' : 'opacity-100 saturate-100'
      }`}
    >
      <Card.Header>
        <Card.Title>Oscilloscope</Card.Title>
        <Switch
          checked={Boolean(!bypassParam?.value)}
          size="sm"
          onChange={(checked) => {
            setBypassValue(!checked);
          }}
        />
      </Card.Header>
      <Card.Content>
        <OscilloscopeView connected={connected} frame={frame} />
      </Card.Content>
    </Card>
  );
}
