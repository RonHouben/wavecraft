import { ErrorMessage, Knob, RadioGroup, Switch } from '@wavecraft/components';
import { ProcessorCard } from '@wavecraft/components/processors/ProcessorCard';
import { ParameterId } from '@wavecraft/core';
import {
  type ParameterInfo,
  ParameterValue,
  type ProcessorId,
  logger,
  useParametersForProcessor,
} from '@wavecraft/core';
import type { ComponentPropsWithoutRef, ElementType, JSX } from 'react';
import { useMemo } from 'react';

export interface SmartProcessorProps<T extends ElementType> {
  readonly processorId: ProcessorId;
  readonly bypassParameterId: ParameterId;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly title: string;
  readonly radioGroupOptions?: {
    renderOptionsAs?: T;
  } & Omit<ComponentPropsWithoutRef<T>, 'children'>;
}

function isBypassParameter<T extends ParameterValue>(param: Pick<ParameterInfo<T>, 'id'>): boolean {
  return param.id.endsWith('_bypass');
}

export function SmartProcessor<T extends ElementType>(
  props: Readonly<SmartProcessorProps<T>>
): JSX.Element | null {
  const { params, isLoading, error, setParameter } = useParametersForProcessor(props.processorId);
  const { renderOptionsAs, ...radioGroupOptionProps } = props.radioGroupOptions ?? {};

  const processorParameters = useMemo(() => {
    const mappedParameters = params.map((param) => ({
      ...param,
      onChange: async (value: ParameterValue): Promise<void> => {
        try {
          await setParameter(param.id, value);
        } catch (err) {
          logger.error('Failed to set processor parameter', {
            error: err,
            parameterId: param.id,
            processorId: props.processorId,
          });
        }
      },
    }));

    // Bypass is arranged separately
    return mappedParameters.filter((param) => !isBypassParameter(param));
  }, [props.processorId, params, setParameter]);

  if (isLoading) {
    return (
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="rounded-xl border border-plugin-border bg-plugin-surface-1 p-5 text-type-sm italic text-plugin-text-muted shadow-panel"
      >
        Loading {props.processorId}...
      </div>
    );
  }

  if (error) {
    return <ErrorMessage message={`Error loading ${props.processorId}: ${error.message}`} />;
  }

  if (processorParameters.length === 0) {
    return null;
  }

  return (
    <ProcessorCard
      bypassParameterId={props.bypassParameterId}
      title={props.title}
      hideWhenNotInSignalChain={props.hideWhenNotInSignalChain}
    >
      {processorParameters.map((param) => {
        switch (param.type) {
          case 'bool':
            return (
              <Switch
                key={param.id}
                checked={Boolean(param.value)}
                onChange={(value) => param.onChange(value)}
              />
            );

          case 'enum': {
            const enumValue = typeof param.value === 'number' ? param.value : Number(param.value);
            const enumOptions = (param.variants ?? []).map((variant, index) => ({
              as: renderOptionsAs,
              ...radioGroupOptionProps,
              label: variant ?? `Option ${index + 1}`,
              value: index,
              children: variant,
            }));

            if (enumOptions.length === 0) {
              return (
                <ErrorMessage
                  key={param.id}
                  message={`Error: Enum parameter has no variants: ${param.name}`}
                />
              );
            }

            return (
              <RadioGroup
                key={param.id}
                name={param.id}
                value={enumValue}
                onChange={(newValue) => param.onChange(newValue)}
                options={enumOptions}
                orientation="vertical"
                label={param.name}
              />
            );
          }
          case 'float':
            return (
              <Knob key={param.id} {...param} label={param.name} value={Number(param.value)} />
            );
          default:
            return (
              <ErrorMessage
                key={param.id}
                message={`Error: Unsupported parameter type: ${param.type}`}
              />
            );
        }
      })}
    </ProcessorCard>
  );
}
