import { useMemo } from 'react';

import { PROCESSOR_BYPASS_SUFFIX } from '../processors/bypass';
import type {
  ParameterId,
  ParameterInfo,
  ParameterValue,
  ParameterVariant,
} from '../types/parameters';
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

function selectProcessorParams<
  T extends ParameterValue = ParameterValue,
  V extends ParameterVariant = ParameterVariant,
>(allParams: readonly ParameterInfo<T, V>[], processorId: ProcessorId): ParameterInfo<T, V>[] {
  const bypassId = `${processorId}${PROCESSOR_BYPASS_SUFFIX}`;

  return allParams.filter(
    (param) => param.id === bypassId || param.id.startsWith(`${processorId}_`)
  );
}

export function useParametersForProcessor(
  processorId: ProcessorId
): UseParametersForProcessorResult<ParameterValue> {
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
