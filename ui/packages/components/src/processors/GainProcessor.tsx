import type { LevelProcessorId } from '@wavecraft/core';
import { useParameter } from '@wavecraft/core';
import { type JSX } from 'react';
import { Col } from '../Col';
import { Knob } from '../Knob';
import { Row } from '../Row';
import { mergeClassNames } from '../utils/classNames';
import { ProcessorCard } from './ProcessorCard';

type GainLikeProcessorId = LevelProcessorId;

export interface GainProcessorProps {
  readonly processorId: GainLikeProcessorId;
  readonly title: string;
  readonly subtitle: string;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

export function GainProcessor({
  processorId,
  title,
  subtitle,
  hideWhenNotInSignalChain,
  className,
}: Readonly<GainProcessorProps>): JSX.Element | null {
  const { param: levelParameter, setValue: setLevelValue } = useParameter<number>(
    `${processorId}_level`
  );

  return (
    <ProcessorCard
      processorId={processorId}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      title={title}
      subtitle={subtitle}
      className={mergeClassNames('h-full w-full', className)}
    >
      <Row className="flex-nowrap items-start gap-1.5">
        <Col className="h-auto items-center justify-start self-start">
          <Knob
            id={`param-${levelParameter?.id}`}
            label={levelParameter?.name ?? ''}
            value={levelParameter?.value ?? 0}
            defaultValue={levelParameter?.default ?? levelParameter?.min ?? 0}
            min={levelParameter?.min ?? 0}
            max={levelParameter?.max ?? 0}
            unit={levelParameter?.unit ?? ''}
            size="sm"
            onChange={setLevelValue}
          />
        </Col>
      </Row>
    </ProcessorCard>
  );
}
