import { useCallback } from 'react';

import { getProcessorBypassParamId } from '../processors/bypass';
import type { ProcessorId } from '../types/processors';
import { useParameter } from './useParameter';

export interface UseProcessorBypassResult {
  processorId: ProcessorId;
  bypassParameterId: string;
  bypassed: boolean;
  loading: boolean;
  error: Error | null;
  setBypassed: (nextBypassed: boolean) => Promise<void>;
  toggle: () => Promise<void>;
}

export function useProcessorBypass(processorId: ProcessorId): UseProcessorBypassResult {
  const bypassParameterId = getProcessorBypassParamId(processorId);
  const { param, setValue, isLoading, error } = useParameter(bypassParameterId);

  const bypassed = Boolean(param?.value);

  const setBypassed = useCallback(
    async (nextBypassed: boolean) => {
      await setValue(nextBypassed);
    },
    [setValue]
  );

  const toggle = useCallback(async () => {
    await setValue(!bypassed);
  }, [bypassed, setValue]);

  return {
    processorId,
    bypassParameterId,
    bypassed,
    loading: isLoading,
    error,
    setBypassed,
    toggle,
  };
}
