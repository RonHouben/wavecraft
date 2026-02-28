import type { SoftClipParameterIds, SoftClipProcessorId } from '@wavecraft/core';
import { useParameter } from '@wavecraft/core';
import { type JSX } from 'react';
import { Col } from '../Col';
import { Knob } from '../Knob';
import { Row } from '../Row';
import { mergeClassNames } from '../utils/classNames';
import { ProcessorCard } from './ProcessorCard';

const SATURATOR_PROCESSOR_ID: SoftClipProcessorId = 'soft_clip';

const saturatorParameterIds: SoftClipParameterIds = {
  bypass: `${SATURATOR_PROCESSOR_ID}_bypass`,
  driveDb: `${SATURATOR_PROCESSOR_ID}_drive_db`,
  outputDb: `${SATURATOR_PROCESSOR_ID}_output_db`,
  mix: `${SATURATOR_PROCESSOR_ID}_mix`,
  tone: `${SATURATOR_PROCESSOR_ID}_tone`,
};

export interface SaturatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

export function SaturatorProcessor({
  hideWhenNotInSignalChain,
  className,
}: Readonly<SaturatorProcessorProps>): JSX.Element | null {
  const { param: driveParameter, setValue: setDriveValue } = useParameter<number>(
    saturatorParameterIds.driveDb
  );
  const { param: outputParameter, setValue: setOutputValue } = useParameter<number>(
    saturatorParameterIds.outputDb
  );
  const { param: mixParameter, setValue: setMixValue } = useParameter<number>(
    saturatorParameterIds.mix
  );
  const { param: toneParameter, setValue: setToneValue } = useParameter<number>(
    saturatorParameterIds.tone
  );

  return (
    <ProcessorCard
      processorId={SATURATOR_PROCESSOR_ID}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      subtitle="Warm Soft Clip"
      title="Saturator"
      className={mergeClassNames('h-full w-full', className)}
    >
      <Row className="flex flex-wrap gap-2">
        <Knob
          id={`param-${driveParameter?.id ?? saturatorParameterIds.driveDb}`}
          label={driveParameter?.name ?? 'Drive'}
          value={driveParameter?.value ?? 0}
          defaultValue={driveParameter?.default ?? 0}
          min={driveParameter?.min ?? 0}
          max={driveParameter?.max ?? 0}
          unit={driveParameter?.unit ?? ''}
          size="sm"
          onChange={setDriveValue}
        />
        <Knob
          id={`param-${outputParameter?.id ?? saturatorParameterIds.outputDb}`}
          label={outputParameter?.name ?? 'Output'}
          value={outputParameter?.value ?? 0}
          defaultValue={outputParameter?.default ?? 0}
          min={outputParameter?.min ?? 0}
          max={outputParameter?.max ?? 0}
          unit={outputParameter?.unit ?? ''}
          size="sm"
          onChange={setOutputValue}
        />
        <Knob
          id={`param-${mixParameter?.id ?? saturatorParameterIds.mix}`}
          label={mixParameter?.name ?? 'Mix'}
          value={mixParameter?.value ?? 0}
          defaultValue={mixParameter?.default ?? 0}
          min={mixParameter?.min ?? 0}
          max={mixParameter?.max ?? 0}
          unit={mixParameter?.unit ?? ''}
          size="sm"
          onChange={setMixValue}
        />
        <Knob
          id={`param-${toneParameter?.id ?? saturatorParameterIds.tone}`}
          label={toneParameter?.name ?? 'Tone'}
          value={toneParameter?.value ?? 0}
          defaultValue={toneParameter?.default ?? 0}
          min={toneParameter?.min ?? 0}
          max={toneParameter?.max ?? 0}
          unit={toneParameter?.unit ?? ''}
          size="sm"
          onChange={setToneValue}
        />
      </Row>
    </ProcessorCard>
  );
}
