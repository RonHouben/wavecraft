import { Col, Knob, Row, Switch } from '@wavecraft/components';
import { useHasProcessorInSignalChain, useParameter } from '@wavecraft/core';
import { type JSX } from 'react';

export interface TestToneProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

export function TestToneProcessor({
  hideWhenNotInSignalChain,
}: Readonly<TestToneProcessorProps>): JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('test_tone');

  const { param: bypassParameter, setValue: setBypassValue } =
    useParameter<boolean>('test_tone_bypass');
  const { param: enabledParameter, setValue: setEnabledValue } =
    useParameter<boolean>('test_tone_enabled');
  const { param: frequencyParameter, setValue: setFrequencyValue } =
    useParameter<number>('test_tone_frequency');
  const { param: levelParameter, setValue: setLevelValue } =
    useParameter<number>('test_tone_level');

  if (hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  return (
    <section
      data-bypassed={bypassParameter?.value}
      className={`w-fit rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel transition-[opacity,filter] duration-150 ${
        bypassParameter?.value ? 'opacity-70 brightness-90 saturate-50' : 'opacity-100 saturate-100'
      }`}
    >
      <header className="mb-3 flex flex-col items-start justify-between gap-3 border-plugin-border/70 pb-3">
        <Row className="justify-between">
          <Col className="justify-between gap-2">
            <h3 className="truncate text-type-md font-semibold text-plugin-text-primary">
              Test Tone
            </h3>
            <h5 className="text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
              Test Signal
            </h5>
          </Col>
          <Col className="justify-start self-start">
            <Switch
              id={`param-${enabledParameter?.id}-switch`}
              checked={Boolean(!bypassParameter?.value)}
              size="sm"
              onChange={(checked) => {
                setBypassValue(!checked);
                setEnabledValue(checked);
              }}
            />
          </Col>
        </Row>
      </header>

      <main>
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
      </main>
    </section>
  );
}
