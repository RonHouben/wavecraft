import { ParameterId, useHasProcessorInSignalChain, useParameter } from '@wavecraft/core';
import { Card, Switch } from '..';

export interface ProcessorCardProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly bypassParameterId: ParameterId;
  readonly title: string;
  readonly children: React.ReactNode;
}

export function ProcessorCard(props: Readonly<ProcessorCardProps>): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscilloscope_tap');

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
      <Card.Header className="">
        <Card.Title>{props.title}</Card.Title>
        <Switch
          checked={Boolean(!bypassParam?.value)}
          size="sm"
          onChange={(checked) => {
            setBypassValue(!checked);
          }}
        />
      </Card.Header>
      <Card.Content>{props.children}</Card.Content>
    </Card>
  );
}
