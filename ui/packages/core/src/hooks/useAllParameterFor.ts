import { useMemo } from 'react';

import { PROCESSOR_BYPASS_SUFFIX } from '../processors/bypass';
import type { ParameterInfo } from '../types/parameters';
import type { ProcessorId } from '../types/processors';
import { useAllParameters } from './useAllParameters';

export interface UseAllParameterForResult {
  processorId: ProcessorId;
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  reload: () => Promise<void>;
}

function selectProcessorParams(
  allParams: readonly ParameterInfo[],
  processorId: ProcessorId
): ParameterInfo[] {
  const bypassId = `${processorId}${PROCESSOR_BYPASS_SUFFIX}`;

  return allParams.filter(
    (param) => param.id === bypassId || param.id.startsWith(`${processorId}_`)
  );
}

export function useAllParametersFor(processorId: ProcessorId): UseAllParameterForResult {
  const { params, isLoading, error, reload } = useAllParameters();

  const processorParams = useMemo(
    () => selectProcessorParams(params, processorId),
    [params, processorId]
  );

  return {
    processorId,
    params: processorParams,
    isLoading,
    error,
    reload,
  };
}
