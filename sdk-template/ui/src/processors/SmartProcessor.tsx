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
import type { JSX } from 'react';
import { useMemo } from 'react';

export interface SmartProcessorProps {
  readonly processorId: ProcessorId;
  readonly bypassParameterId: ParameterId;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly title: string;
}

function isBypassParameter<T extends ParameterValue>(param: Pick<ParameterInfo<T>, 'id'>): boolean {
  return param.id.endsWith('_bypass');
}

export function SmartProcessor(props: Readonly<SmartProcessorProps>): JSX.Element | null {
  const { params, isLoading, error, setParameter } = useParametersForProcessor(props.processorId);

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
              <Switch checked={Boolean(param.value)} onChange={(value) => param.onChange(value)} />
            );

          case 'enum':
            {
              const enumValue = typeof param.value === 'number' ? param.value : Number(param.value);
              const enumOptions = (param.variants ?? []).map((variant, index) => ({
                label: variant ?? `Option ${index + 1}`,
                value: index,
              }));

              if (enumOptions.length === 0) {
                return <ErrorMessage message={`Error: Enum parameter has no variants: ${param.name}`} />;
              }

              return (
                <RadioGroup
                  name={param.id}
                  value={enumValue}
                  onChange={(newValue) => param.onChange(newValue)}
                  options={enumOptions}
                ></RadioGroup>
              );
            }
          case 'float':
            return <Knob {...param} label={param.name} value={Number(param.value)} />;
          default:
            return <ErrorMessage message={`Error: Unsupported parameter type: ${param.type}`} />;
        }
      })}
    </ProcessorCard>
  );
}
