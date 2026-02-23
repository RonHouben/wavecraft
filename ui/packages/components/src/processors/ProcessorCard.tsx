import {
  type ParameterId,
  type ProcessorId,
  useHasProcessorInSignalChain,
  useParameter,
} from '@wavecraft/core';
import { Card, Switch } from '..';

export interface ProcessorCardProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly signalChainProcessorId?: ProcessorId;
  readonly bypassParameterId: ParameterId;
  readonly switchId?: string;
  readonly subtitle?: string;
  readonly title: string;
  readonly onSwitchChange?: (checked: boolean) => void;
  readonly children: React.ReactNode;
}

export function ProcessorCard(props: Readonly<ProcessorCardProps>): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(
    props.signalChainProcessorId ?? 'oscilloscope_tap'
  );

  const { param: bypassParam, setValue: setBypassValue } = useParameter(props.bypassParameterId);

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
        <div>
          <Card.Title>{props.title}</Card.Title>
          {props.subtitle ? (
            <p className="text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
              {props.subtitle}
            </p>
          ) : null}
        </div>
        <Switch
          id={props.switchId}
          checked={Boolean(!bypassParam?.value)}
          size="sm"
          onChange={(checked) => {
            setBypassValue(!checked);
            props.onSwitchChange?.(checked);
          }}
        />
      </Card.Header>
      <Card.Content>{props.children}</Card.Content>
    </Card>
  );
}
