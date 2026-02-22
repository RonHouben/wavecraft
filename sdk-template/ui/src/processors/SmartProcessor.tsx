import {
  type ParameterInfo,
  type ProcessorId,
  logger,
  useParametersForProcessor,
  useHasProcessorInSignalChain,
} from '@wavecraft/core';
import { Button, Fader, Knob, Toggle } from '@wavecraft/components';
import { useMemo } from 'react';
import type { JSX } from 'react';

export interface SmartProcessorProps {
  readonly id: ProcessorId;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly title?: string;
}

interface SmartProcessorParameter extends ParameterInfo {
  readonly onChange: (value: number | boolean) => Promise<void>;
  readonly disabled?: boolean;
}

function isFaderParameter(param: Pick<ParameterInfo, 'id' | 'name' | 'unit'>): boolean {
  const faderHintRegex = /(level|gain|trim|volume|db)/i;
  return faderHintRegex.test(`${param.id} ${param.name} ${param.unit ?? ''}`);
}

function getNumericValue(value: number | boolean): number {
  return typeof value === 'number' ? value : value ? 1 : 0;
}

function renderPrimitiveParameter(param: SmartProcessorParameter): JSX.Element | null {
  const controlId = `param-${param.id}`;

  if (param.type === 'bool') {
    return (
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-surface p-3">
        <Toggle
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
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-surface p-3">
        <p className="mb-2 text-sm font-semibold text-plugin-text-primary">{param.name}</p>
        {variants.length > 0 ? (
          <div role="group" aria-label={param.name} className="flex flex-wrap gap-2">
            {variants.map((variant, index) => (
              <Button
                key={`${param.id}-${variant}-${index}`}
                size="sm"
                pressed={index === selectedIndex}
                disabled={param.disabled}
                onClick={(): void => {
                  void param.onChange(index);
                }}
              >
                {variant}
              </Button>
            ))}
          </div>
        ) : (
          <p className="text-plugin-text-muted text-xs">No variants available</p>
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
      <div key={param.id} className="rounded-md border border-plugin-border bg-plugin-surface p-3">
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
    );
  }

  return null;
}

export function SmartProcessor({
  id,
  hideWhenNotInSignalChain = false,
  title,
}: Readonly<SmartProcessorProps>): JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params, isLoading, error, setParameter } = useParametersForProcessor(id);

  const processorParameters: SmartProcessorParameter[] = useMemo(
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
              processorId: id,
            });
          }
        },
      })),
    [id, params, setParameter]
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
        className="text-plugin-text-muted rounded-lg border border-plugin-border bg-plugin-surface p-4 text-sm italic"
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
        className="rounded-lg border border-state-danger/60 bg-plugin-surface p-4 text-sm text-state-danger"
      >
        Error loading {id}: {error.message}
      </div>
    );
  }

  if (processorParameters.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2 rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <h3 className="text-plugin-text-secondary text-sm font-semibold uppercase tracking-wider">
        {title ?? id}
      </h3>

      <div className="space-y-3">{processorParameters.map((param) => renderPrimitiveParameter(param))}</div>
    </div>
  );
}
