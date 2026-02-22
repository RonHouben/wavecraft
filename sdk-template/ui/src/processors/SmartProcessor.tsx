import {
  type ParameterInfo,
  type ProcessorId,
  logger,
  useParametersForProcessor,
  useHasProcessorInSignalChain,
} from '@wavecraft/core';
import { Button, Fader, Knob, Switch } from '@wavecraft/components';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';
import { ParameterValue } from '@wavecraft/core';

export interface SmartProcessorProps {
  readonly id: ProcessorId;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly title?: string;
}

interface SmartProcessorParameter<T extends ParameterValue> extends ParameterInfo<T> {
  readonly onChange: (value: number | boolean) => Promise<void>;
  readonly disabled?: boolean;
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

function isBypassParameter<T extends ParameterValue>(param: Pick<ParameterInfo<T>, 'id'>): boolean {
  return param.id.endsWith('_bypass');
}

function isFaderParameter<T extends ParameterValue>(
  param: Pick<ParameterInfo<T>, 'id' | 'name' | 'unit'>
): boolean {
  const faderHintRegex = /(level|gain|trim|volume|db)/i;
  return faderHintRegex.test(`${param.id} ${param.name} ${param.unit ?? ''}`);
}

function getNumericValue(value: number | boolean): number {
  return typeof value === 'number' ? value : value ? 1 : 0;
}

function formatParameterValue(
  value: number | boolean,
  unit?: string,
  variants?: readonly string[]
): string {
  if (typeof value === 'boolean') {
    return value ? 'On' : 'Off';
  }

  if (Array.isArray(variants) && variants.length > 0) {
    const variant = variants[getNumericValue(value)] ?? variants[0];
    return variant;
  }

  if (!unit) {
    return value.toFixed(3);
  }

  if (unit === '%') {
    return `${(value * 100).toFixed(1)}%`;
  }

  return `${value.toFixed(2)} ${unit}`;
}

function renderPrimitiveParameter<T extends ParameterValue>(
  param: SmartProcessorParameter<T>
): JSX.Element | null {
  const controlId = `param-${param.id}`;

  if (param.type === 'bool') {
    return (
      <div
        key={param.id}
        className="relative overflow-hidden rounded-xl border border-plugin-border bg-plugin-surface-2/60 p-4 shadow-control backdrop-blur-sm"
      >
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-plugin-border-strong/70 to-transparent"
        />
        <div className="mb-3 flex items-center justify-between gap-3 border-b border-plugin-border/70 pb-3">
          <p className="text-type-sm font-semibold text-plugin-text-primary">{param.name}</p>
          <span className="rounded-full border border-accent/35 bg-gradient-to-b from-accent/20 to-accent/10 px-2.5 py-1 font-mono text-type-xs uppercase tracking-wide text-accent-light">
            {formatParameterValue(Boolean(param.value))}
          </span>
        </div>
        <Switch
          id={controlId}
          label={param.name}
          checked={Boolean(param.value)}
          disabled={param.disabled}
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
      <div
        key={param.id}
        className="relative overflow-hidden rounded-xl border border-plugin-border bg-plugin-surface-2/60 p-4 shadow-control backdrop-blur-sm"
      >
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-plugin-border-strong/70 to-transparent"
        />
        <div className="mb-3 flex flex-wrap items-center justify-between gap-2 border-b border-plugin-border/70 pb-3">
          <p className="text-type-sm font-semibold text-plugin-text-primary">{param.name}</p>
          <span className="rounded-full border border-accent/35 bg-gradient-to-b from-accent/20 to-accent/10 px-2.5 py-1 font-mono text-type-xs uppercase tracking-wide text-accent-light">
            {formatParameterValue(param.value, undefined, variants)}
          </span>
        </div>
        {variants.length > 0 ? (
          <div role="group" aria-label={param.name} className="flex flex-wrap gap-2">
            {variants.map((variant, index) => (
              <Button
                key={`${param.id}-${variant}-${index}`}
                size="sm"
                active={index === selectedIndex}
                disabled={param.disabled}
                className="bg-plugin-dark/70"
                onClick={(): void => {
                  void param.onChange(index);
                }}
              >
                {variant}
              </Button>
            ))}
          </div>
        ) : (
          <p className="text-xs text-plugin-text-muted">No variants available</p>
        )}
      </div>
    );
  }

  if (param.type === 'float') {
    const controlValue = getNumericValue(param.value);
    const onChange = (nextValue: number): void => {
      void param.onChange(nextValue);
    };

    return (
      <div
        key={param.id}
        className="relative overflow-hidden rounded-xl border border-plugin-border bg-plugin-surface-2/60 p-4 shadow-control backdrop-blur-sm"
      >
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-plugin-border-strong/70 to-transparent"
        />
        <div className="mb-3 flex flex-wrap items-center justify-between gap-2 border-b border-plugin-border/70 pb-3">
          <p className="text-type-sm font-semibold text-plugin-text-primary">{param.name}</p>
          <span className="rounded-full border border-plugin-border-strong/80 bg-plugin-dark/70 px-2.5 py-1 font-mono text-type-xs tabular-nums text-plugin-text-primary">
            {formatParameterValue(controlValue, param.unit)}
          </span>
        </div>

        <div className="rounded-lg border border-plugin-border/80 bg-plugin-dark/65 px-3 py-2 shadow-inner">
          {isFaderParameter(param) ? (
            <Fader
              id={controlId}
              label={param.name}
              value={controlValue}
              min={param.min}
              max={param.max}
              unit={param.unit}
              disabled={param.disabled}
              size="lg"
              orientation="horizontal"
              onChange={onChange}
            />
          ) : (
            <Knob
              id={controlId}
              label={param.name}
              value={controlValue}
              min={param.min}
              max={param.max}
              unit={param.unit}
              disabled={param.disabled}
              onChange={onChange}
            />
          )}
        </div>
      </div>
    );
  }

  return null;
}

export function SmartProcessor({
  id,
  hideWhenNotInSignalChain = false,
  title,
}: Readonly<SmartProcessorProps>): JSX.Element | null {
  const sectionRef = useRef<HTMLElement | null>(null);
  const [isPrecisionHintVisible, setIsPrecisionHintVisible] = useState(false);
  const [isPrecisionHintActive, setIsPrecisionHintActive] = useState(false);
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params, isLoading, error, setParameter } = useParametersForProcessor(id);

  const processorParameters = useMemo(() => {
    const mappedParameters = params.map((param) => ({
      ...param,
      onChange: async (value: number | boolean): Promise<void> => {
        try {
          await setParameter(param.id, value);
        } catch (err) {
          logger.error('Failed to set processor parameter', {
            error: err,
            parameterId: param.id,
            processorId: id,
          });
        }
      },
    }));

    const bypassParameters = mappedParameters.filter((param) => isBypassParameter(param));
    const regularParameters = mappedParameters.filter((param) => !isBypassParameter(param));

    return [...bypassParameters, ...regularParameters];
  }, [id, params, setParameter]);

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
        className="rounded-xl border border-plugin-border bg-plugin-surface-1 p-5 text-type-sm italic text-plugin-text-muted shadow-panel"
      >
        Loading {id}...
      </div>
    );
  }

  if (error) {
    return (
      <div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        className="rounded-xl border border-state-danger/60 bg-plugin-surface-1 p-5 text-type-sm text-state-danger shadow-panel"
      >
        Error loading {id}: {error.message}
      </div>
    );
  }

  if (processorParameters.length === 0) {
    return null;
  }

  return (
    <section
      ref={sectionRef}
      className="relative w-fit overflow-hidden rounded-xl border border-plugin-border bg-plugin-surface-1 p-5 shadow-panel"
    >
      <div
        aria-hidden="true"
        className="from-accent/12 pointer-events-none absolute inset-x-0 top-0 h-24 bg-gradient-to-b to-transparent"
      />
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-5 top-0 h-px bg-gradient-to-r from-transparent via-plugin-border-strong/70 to-transparent"
      />

      <div className="relative mb-4 flex items-start justify-between gap-3 border-b border-plugin-border/70 pb-4">
        <div className="min-w-0">
          <p className="mb-1 text-type-2xs uppercase tracking-wider text-plugin-text-muted">
            Processor
          </p>
          <h3 className="truncate text-type-lg font-semibold text-plugin-text-primary">
            {title ?? id}
          </h3>
        </div>

        <span className="rounded-full border border-plugin-border-strong/80 bg-plugin-dark/75 px-2.5 py-1 font-mono text-type-xs tabular-nums text-plugin-text-primary">
          {processorParameters.length} params
        </span>
      </div>

      <p
        data-testid="processor-precision-hint"
        className="relative mb-3 h-4 text-type-xs text-plugin-text-secondary"
      >
        {isPrecisionHintVisible
          ? isPrecisionHintActive
            ? SHIFT_HINT_ACTIVE_TEXT
            : SHIFT_HINT_INACTIVE_TEXT
          : '\u00A0'}
      </p>

      <div className="relative space-y-3">
        {processorParameters.map((param) => renderPrimitiveParameter(param))}
      </div>
    </section>
  );
}
