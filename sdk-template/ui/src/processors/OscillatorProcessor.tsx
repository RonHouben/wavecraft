import {
  type ParameterInfo,
  logger,
  useHasProcessorInSignalChain,
  useParametersForProcessor,
} from '@wavecraft/core';
import { Button, Fader, Knob, Toggle } from '@wavecraft/components';
import { useMemo } from 'react';
import type { JSX } from 'react';

// Test discoverability: this template implementation is validated in
// ui/packages/components/src/OscillatorProcessor.template.test.tsx.

export interface OscillatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

interface OscillatorParameter extends ParameterInfo {
  readonly onChange: (value: number | boolean) => Promise<void>;
}

function hasSuffix(paramId: string, suffix: string): boolean {
  return paramId.endsWith(suffix);
}

function getNumericValue(value: number | boolean): number {
  return typeof value === 'number' ? value : value ? 1 : 0;
}

function formatNumericValue(value: number, unit?: string): string {
  if (!unit) {
    return value.toFixed(3);
  }

  if (unit === '%') {
    return `${(value * 100).toFixed(1)}%`;
  }

  return `${value.toFixed(2)} ${unit}`;
}

function formatEnumValue(param: Pick<ParameterInfo, 'value' | 'variants'>): string {
  const variants = param.variants ?? [];
  if (variants.length === 0) {
    return String(param.value);
  }

  return variants[getNumericValue(param.value)] ?? variants[0] ?? String(param.value);
}

function renderAuxiliaryParameter(param: OscillatorParameter): JSX.Element | null {
  const controlId = `param-${param.id}`;

  if (param.type === 'bool') {
    return (
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
        <Toggle
          id={controlId}
          label={param.name}
          checked={Boolean(param.value)}
          onChange={(checked): void => {
            void param.onChange(checked);
          }}
        />
      </div>
    );
  }

  if (param.type === 'enum') {
    const variants = param.variants ?? [];
    const selectedIndex = getNumericValue(param.value);

    return (
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
        <div className="mb-2 flex items-center justify-between gap-2">
          <span className="text-type-xs uppercase tracking-wide text-plugin-text-secondary">
            {param.name}
          </span>
          <span className="font-mono text-type-xs text-plugin-text-primary">
            {formatEnumValue(param)}
          </span>
        </div>
        <div role="group" aria-label={param.name} className="flex flex-wrap gap-1.5">
          {variants.map((variant, index) => (
            <Button
              key={`${param.id}-${variant}-${index}`}
              size="sm"
              active={index === selectedIndex}
              onClick={(): void => {
                void param.onChange(index);
              }}
            >
              {variant}
            </Button>
          ))}
        </div>
      </div>
    );
  }

  if (param.type === 'float') {
    const numericValue = getNumericValue(param.value);

    return (
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
        <div className="mb-2 flex items-center justify-between gap-2">
          <span className="text-type-xs uppercase tracking-wide text-plugin-text-secondary">
            {param.name}
          </span>
          <span className="font-mono text-type-xs tabular-nums text-plugin-text-primary">
            {formatNumericValue(numericValue, param.unit)}
          </span>
        </div>
        <Fader
          id={controlId}
          label={param.name}
          value={numericValue}
          min={param.min}
          max={param.max}
          unit={param.unit}
          size="sm"
          orientation="horizontal"
          onChange={(nextValue): void => {
            void param.onChange(nextValue);
          }}
        />
      </div>
    );
  }

  return null;
}

export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscillatorProcessorProps>): JSX.Element | null {
  const processorId = 'oscillator';
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(processorId);
  const { params, isLoading, error, setParameter } = useParametersForProcessor(processorId);

  const processorParameters: OscillatorParameter[] = useMemo(
    () =>
      params.map((param) => ({
        ...param,
        onChange: async (value: number | boolean): Promise<void> => {
          try {
            await setParameter(param.id, value);
          } catch (err) {
            logger.error('Failed to set processor parameter', {
              error: err,
              parameterId: param.id,
              processorId,
            });
          }
        },
      })),
    [params, setParameter]
  );

  const bypassParameter = processorParameters.find((param) => hasSuffix(param.id, '_bypass'));
  const enabledParameter = processorParameters.find((param) => hasSuffix(param.id, '_enabled'));
  const waveformParameter = processorParameters.find((param) => hasSuffix(param.id, '_waveform'));
  const frequencyParameter = processorParameters.find(
    (param) => hasSuffix(param.id, '_frequency') && param.type === 'float'
  );
  const levelParameter = processorParameters.find(
    (param) => hasSuffix(param.id, '_level') && param.type === 'float'
  );

  const knownParameterIds = new Set([
    bypassParameter?.id,
    enabledParameter?.id,
    waveformParameter?.id,
    frequencyParameter?.id,
    levelParameter?.id,
  ]);

  const auxiliaryParameters = processorParameters.filter(
    (param) => !knownParameterIds.has(param.id)
  );

  if (hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  if (isLoading) {
    return (
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="rounded-xl border border-plugin-border bg-plugin-surface-1 p-4 text-type-sm italic text-plugin-text-muted shadow-panel"
      >
        Loading oscillator...
      </div>
    );
  }

  if (error) {
    return (
      <div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        className="rounded-xl border border-state-danger/60 bg-plugin-surface-1 p-4 text-type-sm text-state-danger shadow-panel"
      >
        Error loading oscillator: {error.message}
      </div>
    );
  }

  if (processorParameters.length === 0) {
    return null;
  }

  const waveformVariants = waveformParameter?.variants ?? [];
  const selectedWaveform = waveformParameter ? getNumericValue(waveformParameter.value) : 0;

  return (
    <section className="rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel">
      <header className="mb-3 flex items-start justify-between gap-3 border-b border-plugin-border/70 pb-3">
        <div className="min-w-0">
          <p className="text-type-2xs uppercase tracking-wider text-plugin-text-muted">Processor</p>
          <h3 className="truncate text-type-md font-semibold text-plugin-text-primary">
            Oscillator
          </h3>
        </div>

        <div className="flex items-center gap-2">
          {enabledParameter ? (
            <Toggle
              id={`param-${enabledParameter.id}`}
              label="Enabled"
              checked={Boolean(enabledParameter.value)}
              onChange={(checked): void => {
                void enabledParameter.onChange(checked);
              }}
            />
          ) : null}
          {bypassParameter ? (
            <Toggle
              id={`param-${bypassParameter.id}`}
              label="Bypass"
              checked={Boolean(bypassParameter.value)}
              onChange={(checked): void => {
                void bypassParameter.onChange(checked);
              }}
            />
          ) : null}
        </div>
      </header>

      <div className="space-y-3">
        {waveformParameter?.type === 'enum' ? (
          <div className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
            <div className="mb-2 flex items-center justify-between gap-2">
              <span className="text-type-xs uppercase tracking-wide text-plugin-text-secondary">
                {waveformParameter.name}
              </span>
              <span className="font-mono text-type-xs text-plugin-text-primary">
                {formatEnumValue(waveformParameter)}
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
                  onClick={(): void => {
                    void waveformParameter.onChange(index);
                  }}
                >
                  {variant}
                </Button>
              ))}
            </div>
          </div>
        ) : null}

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-[auto_1fr] sm:items-end">
          {frequencyParameter ? (
            <div className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
              <Knob
                id={`param-${frequencyParameter.id}`}
                label={frequencyParameter.name}
                value={getNumericValue(frequencyParameter.value)}
                min={frequencyParameter.min}
                max={frequencyParameter.max}
                unit={frequencyParameter.unit}
                size="sm"
                onChange={(nextValue): void => {
                  void frequencyParameter.onChange(nextValue);
                }}
              />
            </div>
          ) : null}

          {levelParameter ? (
            <div className="rounded-md border border-plugin-border bg-plugin-dark/60 p-2">
              <Fader
                id={`param-${levelParameter.id}`}
                label={levelParameter.name}
                value={getNumericValue(levelParameter.value)}
                min={levelParameter.min}
                max={levelParameter.max}
                unit={levelParameter.unit}
                size="sm"
                orientation="horizontal"
                onChange={(nextValue): void => {
                  void levelParameter.onChange(nextValue);
                }}
              />
            </div>
          ) : null}
        </div>

        {auxiliaryParameters.length > 0 ? (
          <div className="grid grid-cols-1 gap-2">
            {auxiliaryParameters.map(renderAuxiliaryParameter)}
          </div>
        ) : null}
      </div>
    </section>
  );
}
