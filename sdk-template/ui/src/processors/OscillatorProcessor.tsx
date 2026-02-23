import { Col, IconButton, IconProps, Knob, RadioGroup, Row, Switch } from '@wavecraft/components';
import { ParameterInfo } from '@wavecraft/components/types';
import { useHasProcessorInSignalChain, useParameter } from '@wavecraft/core';
import { type JSX } from 'react';

export interface OscillatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

enum WaveformVariant {
  'Sine' = 'Sine',
  'Square' = 'Square',
  'Saw' = 'Saw',
  'Triangle' = 'Triangle',
}

export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscillatorProcessorProps>): JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain('oscillator');

  const { param: bypassParameter, setValue: setBypassValue } =
    useParameter<boolean>('oscillator_bypass');
  const { param: enabledParameter, setValue: setEnabledValue } =
    useParameter<boolean>('oscillator_enabled');
  const { param: waveformParameter, setValue: setWaveformValue } = useParameter<
    number,
    WaveformVariant
  >('oscillator_waveform');
  const { param: frequencyParameter, setValue: setFrequencyValue } =
    useParameter<number>('oscillator_frequency');
  const { param: levelParameter, setValue: setLevelValue } =
    useParameter<number>('oscillator_level');

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
      <header className="mb-3 flex flex-col items-start justify-between gap-3 border-b border-plugin-border/70 pb-3">
        <Row className="justify-between">
          <Col className="justify-between gap-2">
            <h3 className="truncate text-type-md font-semibold text-plugin-text-primary">
              Oscillator
            </h3>
            <h5 className="text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
              Preset Name
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

      <div className="grid gap-3 space-y-3">
        <div className="w-fit rounded-md border border-plugin-border bg-plugin-dark/60 p-2 sm:col-span-4">
          <div className="mb-2">
            <span className="text-type-xs uppercase tracking-wide text-plugin-text-secondary">
              {waveformParameter?.name}
            </span>
          </div>
          <Row role="group" aria-label={waveformParameter?.name} className="gap-1.5">
            <>
              {waveformParameter && (
                <RadioGroup
                  name="waveform"
                  value={mapWaveformValueToVariant(waveformParameter.value, waveformParameter)}
                  options={
                    waveformParameter.variants?.map((variant) => ({
                      as: IconButton,
                      label: variant,
                      value: variant,
                      icon: mapWaveformParameterToIconVariant(variant),
                      // size: 'sm',
                    })) ?? []
                  }
                  onChange={(selectedWaveform) => {
                    if (isWaveformVariant(selectedWaveform)) {
                      const newWaveformValue = mapWaveformVariantToValue(selectedWaveform);

                      console.log({ selectedWaveform, newWaveformValue });

                      setWaveformValue(newWaveformValue);
                    }
                  }}
                />
              )}
            </>
          </Row>
        </div>

        <Row>
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
        </Row>
      </div>
    </section>
  );
}

function mapWaveformValueToVariant(
  waveformValue: number,
  waveformParameters: ParameterInfo<number, WaveformVariant>
): WaveformVariant | undefined {
  return waveformParameters.variants ? waveformParameters.variants[waveformValue] : undefined;
}

function mapWaveformVariantToValue(waveformVariant: WaveformVariant): number {
  return Object.values(WaveformVariant).indexOf(waveformVariant);
}

function mapWaveformParameterToIconVariant(
  waveformParameterValue: WaveformVariant
): IconProps['icon'] {
  switch (waveformParameterValue) {
    case WaveformVariant.Sine:
      return 'waveform-sine';
    case WaveformVariant.Square:
      return 'waveform-square';
    case WaveformVariant.Saw:
      return 'waveform-saw';
    case WaveformVariant.Triangle:
      return 'waveform-triangle';
    default:
      throw new Error(`Unsupported waveform variant: ${waveformParameterValue}`);
  }
}

function isWaveformVariant(value: unknown): value is WaveformVariant {
  return (
    typeof value === 'string' && Object.values(WaveformVariant).includes(value as WaveformVariant)
  );
}
