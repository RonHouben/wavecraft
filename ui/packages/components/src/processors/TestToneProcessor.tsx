import { Col, Knob, Row, Switch } from '..';
import type { TestToneParameterIds, TestToneProcessorId } from '@wavecraft/core';
import { useParameter } from '@wavecraft/core';
import { type JSX } from 'react';
import { ProcessorCard } from './ProcessorCard';
import { mergeClassNames } from '../utils/classNames';

const TEST_TONE_PROCESSOR_ID: TestToneProcessorId = 'test_tone';

const testToneParameterIds: TestToneParameterIds = {
  bypass: `${TEST_TONE_PROCESSOR_ID}_bypass`,
  enabled: `${TEST_TONE_PROCESSOR_ID}_enabled`,
  frequency: `${TEST_TONE_PROCESSOR_ID}_frequency`,
  level: `${TEST_TONE_PROCESSOR_ID}_level`,
};

export interface TestToneProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

export function TestToneProcessor({
  hideWhenNotInSignalChain,
  className,
}: Readonly<TestToneProcessorProps>): JSX.Element | null {
  const { param: enabledParameter, setValue: setEnabledValue } = useParameter<boolean>(
    testToneParameterIds.enabled
  );
  const { param: frequencyParameter, setValue: setFrequencyValue } = useParameter<number>(
    testToneParameterIds.frequency
  );
  const { param: levelParameter, setValue: setLevelValue } = useParameter<number>(
    testToneParameterIds.level
  );

  return (
    <ProcessorCard
      processorId={TEST_TONE_PROCESSOR_ID}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      subtitle="Test Signal"
      title="Test Tone"
      className={mergeClassNames('h-full w-full', className)}
    >
      <Row className="flex-nowrap items-start gap-1.5">
        <Col className="h-auto items-center justify-start self-start">
          <div className="mb-2 flex w-full items-center justify-between gap-2">
            <span className="text-type-xs text-plugin-text-secondary">
              {enabledParameter?.name ?? 'Enabled'}
            </span>
            <Switch
              id={`param-${enabledParameter?.id}-switch`}
              checked={Boolean(enabledParameter?.value)}
              size="sm"
              onChange={setEnabledValue}
            />
          </div>
          <Knob
            id={`param-${frequencyParameter?.id}`}
            label={frequencyParameter?.name ?? ''}
            value={frequencyParameter?.value ?? 0}
            min={frequencyParameter?.min ?? 0}
            max={frequencyParameter?.max ?? 0}
            unit={frequencyParameter?.unit ?? ''}
            size="sm"
            onChange={setFrequencyValue}
          />
          <Knob
            id={`param-${levelParameter?.id}`}
            label={levelParameter?.name ?? ''}
            value={levelParameter?.value ?? 0}
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
