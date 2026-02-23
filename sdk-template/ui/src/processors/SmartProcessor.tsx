import { Switch } from '@wavecraft/components';
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

    const bypassParameters = mappedParameters.filter((param) => isBypassParameter(param));
    const regularParameters = mappedParameters.filter((param) => !isBypassParameter(param));

    return [...bypassParameters, ...regularParameters];
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
    return (
      <div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        className="rounded-xl border border-state-danger/60 bg-plugin-surface-1 p-5 text-type-sm text-state-danger shadow-panel"
      >
        Error loading {props.processorId}: {error.message}
      </div>
    );
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
          default:
            return <div>Error: Unsupported parameter type: {param.type}</div>;
        }
      })}
    </ProcessorCard>
  );
}
