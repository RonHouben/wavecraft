import { Button, Fader, Knob } from '@wavecraft/components';
import { useHasProcessorInSignalChain, useParameter } from '@wavecraft/core';
import { useRef, type JSX } from 'react';

export interface OscillatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

const BYPASS_ACTIVE_CLASS =
  'border-accent-light bg-gradient-to-b from-accent/60 to-accent/35 text-accent-light ring-2 ring-inset ring-accent/70';

function getNumericValue(value: number | boolean): number {
  return typeof value === 'number' ? value : value ? 1 : 0;
}

export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscillatorProcessorProps>): JSX.Element | null {
  const sectionRef = useRef<HTMLElement | null>(null);
  const processorId = 'oscillator';
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(processorId);

  const { param: bypassParameter, setValue: setBypassValue } =
    useParameter<boolean>('oscillator_bypass');
  const { param: enabledParameter, setValue: setEnabledValue } =
    useParameter<boolean>('oscillator_enabled');
  const { param: waveformParameter, setValue: setWaveformValue } =
    useParameter<number>('oscillator_waveform');
  const { param: frequencyParameter, setValue: setFrequencyValue } =
    useParameter<number>('oscillator_frequency');
  const { param: levelParameter, setValue: setLevelValue } =
    useParameter<number>('oscillator_level');

  if (hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const isBypassActive = Boolean(bypassParameter?.value);
  const waveformVariants = waveformParameter?.variants ?? [];
  const selectedWaveform = waveformParameter ? getNumericValue(waveformParameter.value) : 0;

  return (
    <section
      ref={sectionRef}
      data-bypassed={String(isBypassActive)}
      className={`w-fit rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel transition-[opacity,filter] duration-150 ${
        isBypassActive ? 'opacity-70 brightness-90 saturate-50' : 'opacity-100 saturate-100'
      }`}
    >
      <header className="mb-3 flex items-start justify-between gap-3 border-b border-plugin-border/70 pb-3">
        <h3 className="truncate text-type-md font-semibold text-plugin-text-primary">Oscillator</h3>

        <div className="flex items-center gap-2">
          <Button
            id={`param-${bypassParameter?.id}`}
            size="sm"
            active={Boolean(bypassParameter?.value)}
            className={bypassParameter?.value === true ? BYPASS_ACTIVE_CLASS : undefined}
            onClick={() => setBypassValue(!bypassParameter?.value)}
          >
            Bypass
          </Button>
        </div>
      </header>

      <Button
        id={`param-${enabledParameter?.id}`}
        size="sm"
        active={enabledParameter?.value ?? false}
        onClick={() => setEnabledValue(!enabledParameter?.value)}
      >
        Enabled
      </Button>

      <div className="grid gap-3 space-y-3 sm:grid-cols-4 sm:items-end sm:justify-start">
        {waveformParameter?.type === 'enum' ? (
          <div className="w-fit rounded-md border border-plugin-border bg-plugin-dark/60 p-2 sm:col-span-4">
            <div className="mb-2">
              <span className="text-type-xs uppercase tracking-wide text-plugin-text-secondary">
                {waveformParameter.name}
              </span>
            </div>
            <div
              role="group"
              aria-label={waveformParameter.name}
              className="flex flex-wrap gap-1.5"
            >
              {waveformVariants.map((variant, index) => (
                <Button
                  key={`${waveformParameter.id}-${variant}-${index}`}
                  size="sm"
                  active={index === selectedWaveform}
                  onClick={() => setWaveformValue(index)}
                >
                  {variant}
                </Button>
              ))}
            </div>
          </div>
        ) : null}

        <div className="w-fit rounded-md border border-plugin-border bg-plugin-dark/60 p-2 sm:col-span-1">
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
        </div>

        <div className="w-fit rounded-md border border-plugin-border bg-plugin-dark/60 p-2 sm:col-span-1">
          <Fader
            id={`param-${levelParameter?.id}`}
            label={levelParameter?.name ?? ''}
            value={levelParameter?.value ?? 0}
            min={levelParameter?.min ?? 0}
            max={levelParameter?.max ?? 0}
            unit={levelParameter?.unit ?? ''}
            size="sm"
            orientation="vertical"
            onChange={setLevelValue}
          />
        </div>
      </div>
    </section>
  );
}
