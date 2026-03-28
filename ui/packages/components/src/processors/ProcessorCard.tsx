import {
  type BypassProcessorId,
  useHasProcessorInSignalChain,
  useParameter,
} from '@wavecraft/core';
import { Card, Switch } from '..';
import { elevatedCardClass, mergeClassNames } from '../utils/classNames';

export interface ProcessorCardProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly processorId: BypassProcessorId;
  readonly subtitle?: string;
  readonly title: string;
  readonly children?: React.ReactNode;
  readonly className?: string;
}

export function ProcessorCard(props: Readonly<ProcessorCardProps>): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(props.processorId);

  const { param: bypassParam, setValue: setBypassValue } = useParameter(
    `${props.processorId}_bypass`
  );

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return (
    <Card
      data-bypassed={bypassParam?.value}
      className={mergeClassNames(
        'h-full w-full transition-[opacity,filter] duration-150',
        elevatedCardClass,
        bypassParam?.value ? 'opacity-70 brightness-90 saturate-50' : 'opacity-100 saturate-100',
        props.className
      )}
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
          id={`param-${bypassParam?.id}-switch`}
          checked={Boolean(!bypassParam?.value)}
          size="sm"
          onChange={(checked) => {
            setBypassValue(!checked);
          }}
        />
      </Card.Header>
      <Card.Content className="flex flex-col gap-4">{props.children}</Card.Content>
    </Card>
  );
}
