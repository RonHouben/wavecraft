import { useMemo } from 'react';

import { PROCESSOR_BYPASS_SUFFIX } from '../processors/bypass';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';
import type { ProcessorId } from '../types/processors';
import { useAllParameters } from './useAllParameters';

export interface UseParametersForProcessorResult<T extends ParameterValue> {
  processorId: ProcessorId;
  params: ParameterInfo<T>[];
  isLoading: boolean;
  error: Error | null;
  setParameter: (id: ParameterId, value: T) => Promise<void>;
  reload: () => Promise<void>;
}

function selectProcessorParams<T extends ParameterValue>(
  allParams: readonly ParameterInfo<T>[],
  processorId: ProcessorId
): ParameterInfo<T>[] {
  const bypassId = `${processorId}${PROCESSOR_BYPASS_SUFFIX}`;

  return allParams.filter(
    (param) => param.id === bypassId || param.id.startsWith(`${processorId}_`)
  );
}

export function useParametersForProcessor<T extends ParameterValue>(
  processorId: ProcessorId
): UseParametersForProcessorResult<T> {
  const { params, isLoading, error, setParameter, reload } = useAllParameters();

  const processorParams = useMemo(
    () => selectProcessorParams<T>(params, processorId),
    [params, processorId]
  );

  return {
    processorId,
    params: processorParams,
    isLoading,
    error,
    setParameter,
    reload,
  };
}
