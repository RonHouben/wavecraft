import { Col, Knob, Row } from '@wavecraft/components';
import { useParameter } from '@wavecraft/core';
import { type JSX } from 'react';
import { ProcessorCard } from './ProcessorCard';

export interface TestToneProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function TestToneProcessor({
  hideWhenNotInSignalChain,
}: Readonly<TestToneProcessorProps>): JSX.Element | null {
  const { param: enabledParameter, setValue: setEnabledValue } =
    useParameter<boolean>('test_tone_enabled');
  const { param: frequencyParameter, setValue: setFrequencyValue } =
    useParameter<number>('test_tone_frequency');
  const { param: levelParameter, setValue: setLevelValue } =
    useParameter<number>('test_tone_level');

  return (
    <ProcessorCard
      bypassParameterId="test_tone_bypass"
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      signalChainProcessorId="test_tone"
      switchId={`param-${enabledParameter?.id}-switch`}
      subtitle="Test Signal"
      title="Test Tone"
      onSwitchChange={(checked) => {
        setEnabledValue(checked);
      }}
    >
      <Row className="flex-nowrap items-start gap-1.5">
        <Col className="h-auto items-center justify-start self-start">
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
