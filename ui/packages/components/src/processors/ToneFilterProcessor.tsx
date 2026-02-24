import type { ToneFilterParameterIds, ToneFilterProcessorId } from '@wavecraft/core';
import { useParameter } from '@wavecraft/core';
import { type JSX } from 'react';
import { Col } from '../Col';
import { Knob } from '../Knob';
import { Row } from '../Row';
import { Select, type SelectOption } from '../Select';
import { mergeClassNames } from '../utils/classNames';
import { ProcessorCard } from './ProcessorCard';

const TONE_FILTER_PROCESSOR_ID: ToneFilterProcessorId = 'tone_filter';

const toneFilterParameterIds: ToneFilterParameterIds = {
  bypass: `${TONE_FILTER_PROCESSOR_ID}_bypass`,
  mode: `${TONE_FILTER_PROCESSOR_ID}_mode`,
  cutoffHz: `${TONE_FILTER_PROCESSOR_ID}_cutoff_hz`,
  resonanceQ: `${TONE_FILTER_PROCESSOR_ID}_resonance_q`,
};

export interface ToneFilterProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

export function ToneFilterProcessor({
  hideWhenNotInSignalChain,
  className,
}: Readonly<ToneFilterProcessorProps>): JSX.Element | null {
  const { param: modeParameter, setValue: setModeValue } = useParameter<number, string>(
    toneFilterParameterIds.mode
  );
  const { param: cutoffParameter, setValue: setCutoffValue } = useParameter<number>(
    toneFilterParameterIds.cutoffHz
  );
  const { param: resonanceParameter, setValue: setResonanceValue } = useParameter<number>(
    toneFilterParameterIds.resonanceQ
  );

  const modeOptions = getModeOptions(modeParameter?.variants);

  return (
    <ProcessorCard
      processorId={TONE_FILTER_PROCESSOR_ID}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      subtitle="Filter"
      title="Tone Filter"
      className={mergeClassNames('h-full w-full', className)}
    >
      <Row className="flex-nowrap items-start gap-3">
        <Col className="h-auto grow justify-start self-start">
          <Select
            id={`param-${modeParameter?.id ?? toneFilterParameterIds.mode}`}
            label={modeParameter?.name ?? 'Mode'}
            value={modeParameter?.value ?? 0}
            options={modeOptions}
            size="sm"
            onChange={setModeValue}
          />
        </Col>
        <Col className="h-auto items-center justify-start self-start">
          <Row className="flex-nowrap items-start gap-1.5">
            <Knob
              id={`param-${cutoffParameter?.id ?? toneFilterParameterIds.cutoffHz}`}
              label={cutoffParameter?.name ?? ''}
              value={cutoffParameter?.value ?? 0}
              min={cutoffParameter?.min ?? 0}
              max={cutoffParameter?.max ?? 0}
              unit={cutoffParameter?.unit ?? ''}
              size="sm"
              onChange={setCutoffValue}
            />
            <Knob
              id={`param-${resonanceParameter?.id ?? toneFilterParameterIds.resonanceQ}`}
              label={resonanceParameter?.name ?? ''}
              value={resonanceParameter?.value ?? 0}
              min={resonanceParameter?.min ?? 0}
              max={resonanceParameter?.max ?? 0}
              unit={resonanceParameter?.unit ?? ''}
              size="sm"
              onChange={setResonanceValue}
            />
          </Row>
        </Col>
      </Row>
    </ProcessorCard>
  );
}

function getModeOptions(
  variants: readonly (string | undefined)[] | undefined
): readonly SelectOption<number>[] {
  if (!variants || variants.length === 0) {
    return [];
  }

  return variants.reduce<SelectOption<number>[]>((options, label, index) => {
    if (!label) {
      return options;
    }

    options.push({
      label,
      value: index,
    });

    return options;
  }, []);
}
