import { useMemo } from 'react';

import { PROCESSOR_BYPASS_SUFFIX } from '../processors/bypass';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';
import type { ProcessorId } from '../types/processors';
import { useAllParameters } from './useAllParameters';

export interface UseParametersForProcessorResult {
  processorId: ProcessorId;
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  setParameter: (id: ParameterId, value: ParameterValue) => Promise<void>;
  reload: () => Promise<void>;
}

/** @deprecated Use UseParametersForProcessorResult instead. */
export type UseAllParameterForResult = UseParametersForProcessorResult;

function selectProcessorParams(
  allParams: readonly ParameterInfo[],
  processorId: ProcessorId
): ParameterInfo[] {
  const bypassId = `${processorId}${PROCESSOR_BYPASS_SUFFIX}`;

  return allParams.filter(
    (param) => param.id === bypassId || param.id.startsWith(`${processorId}_`)
  );
}

export function useParametersForProcessor(
  processorId: ProcessorId
): UseParametersForProcessorResult {
  const { params, isLoading, error, setParameter, reload } = useAllParameters();

  const processorParams = useMemo(
    () => selectProcessorParams(params, processorId),
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

/** @deprecated Use useParametersForProcessor instead. */
export function useAllParametersFor(processorId: ProcessorId): UseAllParameterForResult {
  return useParametersForProcessor(processorId);
}
