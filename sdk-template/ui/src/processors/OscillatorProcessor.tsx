import {
  type ParameterInfo,
  logger,
  useHasProcessorInSignalChain,
  useParametersForProcessor,
} from '@wavecraft/core';
import { Button, Fader, Knob, Toggle } from '@wavecraft/components';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';

// Test discoverability: this template implementation is validated in
// ui/packages/components/src/OscillatorProcessor.template.test.tsx.

export interface OscillatorProcessorProps {
  readonly hideWhenNotInSignalChain?: boolean;
}

interface OscillatorParameter extends ParameterInfo {
  readonly onChange: (value: number | boolean) => Promise<void>;
}

const SHIFT_HINT_INACTIVE_TEXT = 'Hold Shift for fine adjust';
const SHIFT_HINT_ACTIVE_TEXT = '⇧ Fine adjust';

function getPrecisionControl(target: EventTarget | null): HTMLElement | null {
  if (!(target instanceof HTMLElement)) {
    return null;
  }

  return target.closest('[data-precision-control="true"]');
}

function isPrecisionControlActive(control: HTMLElement | null): boolean {
  return control?.dataset.precisionActive === 'true';
}

function hasSuffix(paramId: string, suffix: string): boolean {
  return paramId.endsWith(suffix);
}

function findCanonicalParameter(
  params: readonly OscillatorParameter[],
  processorId: string,
  suffix: '_bypass' | '_enabled' | '_waveform' | '_frequency' | '_level',
  type?: OscillatorParameter['type']
): OscillatorParameter | undefined {
  const canonicalId = `${processorId}${suffix}`;

  const canonicalMatch = params.find((param) => {
    if (param.id !== canonicalId) {
      return false;
    }

    return type ? param.type === type : true;
  });

  if (canonicalMatch) {
    return canonicalMatch;
  }

  return params.find((param) => {
    if (type && param.type !== type) {
      return false;
    }

    return hasSuffix(param.id, suffix);
  });
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
  const sectionRef = useRef<HTMLElement | null>(null);
  const [isPrecisionHintVisible, setIsPrecisionHintVisible] = useState(false);
  const [isPrecisionHintActive, setIsPrecisionHintActive] = useState(false);
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

  const bypassParameter = findCanonicalParameter(processorParameters, processorId, '_bypass');
  const enabledParameter = findCanonicalParameter(processorParameters, processorId, '_enabled');
  const waveformParameter = findCanonicalParameter(
    processorParameters,
    processorId,
    '_waveform',
    'enum'
  );
  const frequencyParameter = findCanonicalParameter(
    processorParameters,
    processorId,
    '_frequency',
    'float'
  );
  const levelParameter = findCanonicalParameter(
    processorParameters,
    processorId,
    '_level',
    'float'
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

  useEffect(() => {
    const sectionElement = sectionRef.current;

    if (!sectionElement) {
      return;
    }

    const hostElement = sectionElement;

    function syncPrecisionHintState(): void {
      const activeControl = getPrecisionControl(document.activeElement);

      if (activeControl && hostElement.contains(activeControl)) {
        setIsPrecisionHintVisible(true);
        setIsPrecisionHintActive(isPrecisionControlActive(activeControl));
        return;
      }

      setIsPrecisionHintVisible(false);
      setIsPrecisionHintActive(false);
    }

    function handleContextEvent(event: Event): void {
      const precisionControl = getPrecisionControl(event.target);

      if (!precisionControl || !hostElement.contains(precisionControl)) {
        return;
      }

      setIsPrecisionHintVisible(true);

      if (event instanceof KeyboardEvent && event.key === 'Shift') {
        setIsPrecisionHintActive(event.type === 'keydown');
        return;
      }

      setIsPrecisionHintActive(isPrecisionControlActive(precisionControl));
    }

    function handleFocusOut(): void {
      requestAnimationFrame((): void => {
        syncPrecisionHintState();
      });
    }

    hostElement.addEventListener('focusin', handleContextEvent);
    hostElement.addEventListener('focusout', handleFocusOut);
    hostElement.addEventListener('keydown', handleContextEvent);
    hostElement.addEventListener('keyup', handleContextEvent);
    hostElement.addEventListener('pointerdown', handleContextEvent);
    hostElement.addEventListener('pointerup', handleContextEvent);
    hostElement.addEventListener('pointercancel', handleContextEvent);
    hostElement.addEventListener('change', handleContextEvent);
    hostElement.addEventListener('input', handleContextEvent);

    return (): void => {
      hostElement.removeEventListener('focusin', handleContextEvent);
      hostElement.removeEventListener('focusout', handleFocusOut);
      hostElement.removeEventListener('keydown', handleContextEvent);
      hostElement.removeEventListener('keyup', handleContextEvent);
      hostElement.removeEventListener('pointerdown', handleContextEvent);
      hostElement.removeEventListener('pointerup', handleContextEvent);
      hostElement.removeEventListener('pointercancel', handleContextEvent);
      hostElement.removeEventListener('change', handleContextEvent);
      hostElement.removeEventListener('input', handleContextEvent);
    };
  }, []);

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
    <section
      ref={sectionRef}
      className="rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel"
    >
      <header className="mb-3 flex items-start justify-between gap-3 border-b border-plugin-border/70 pb-3">
        <div className="min-w-0">
          <p className="text-type-2xs uppercase tracking-wider text-plugin-text-muted">Processor</p>
          <h3 className="truncate text-type-md font-semibold text-plugin-text-primary">
            Oscillator
          </h3>
        </div>

        <div className="flex items-center gap-2">
          {enabledParameter ? (
            <Button
              id={`param-${enabledParameter.id}`}
              size="sm"
              active={Boolean(enabledParameter.value)}
              onClick={(): void => {
                void enabledParameter.onChange(!enabledParameter.value);
              }}
            >
              Enabled
            </Button>
          ) : null}
          {bypassParameter ? (
            <Button
              id={`param-${bypassParameter.id}`}
              size="sm"
              active={Boolean(bypassParameter.value)}
              onClick={(): void => {
                void bypassParameter.onChange(!bypassParameter.value);
              }}
            >
              Bypass
            </Button>
          ) : null}
        </div>
      </header>

      <p
        data-testid="processor-precision-hint"
        className="mb-3 h-4 text-type-xs text-plugin-text-secondary"
      >
        {isPrecisionHintVisible
          ? isPrecisionHintActive
            ? SHIFT_HINT_ACTIVE_TEXT
            : SHIFT_HINT_INACTIVE_TEXT
          : '\u00A0'}
      </p>

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

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-[auto_auto] sm:items-end sm:justify-start">
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
                orientation="vertical"
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
