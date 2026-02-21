import {
  type ProcessorId,
  logger,
  useParametersForProcessor,
  useHasProcessorInSignalChain,
} from '@wavecraft/core';
import { Processor } from '@wavecraft/components';
import { useMemo } from 'react';
import type { JSX } from 'react';

export interface SmartProcessorProps {
  readonly id: ProcessorId;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly title?: string;
}

export function SmartProcessor({
  id,
  hideWhenNotInSignalChain = false,
  title,
}: Readonly<SmartProcessorProps>): JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params, isLoading, error, setParameter } = useParametersForProcessor(id);

  const processorParameters = useMemo(
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
      <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4 text-sm italic text-gray-500">
        Loading {id}...
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-400 bg-plugin-surface p-4 text-sm text-red-400">
        Error loading {id}: {error.message}
      </div>
    );
  }

  if (processorParameters.length === 0) {
    return null;
  }

  return <Processor id={id} title={title} parameters={processorParameters} />;
}
