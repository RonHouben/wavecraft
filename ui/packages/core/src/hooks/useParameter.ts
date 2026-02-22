/**
 * useParameter - Hook for managing a single parameter
 */

import { useCallback, useMemo, useState } from 'react';

import { useAllParameters } from './useAllParameters';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';

function toError(err: unknown): Error {
  return err instanceof Error ? err : new Error(String(err));
}

export interface UseParameterResult {
  param: ParameterInfo | null;
  setValue: (value: ParameterValue) => Promise<void>;
  isLoading: boolean;
  error: Error | null;
}

function toBackendValue(value: ParameterValue): number {
  return typeof value === 'boolean' ? (value ? 1 : 0) : value;
}

function toFrontendValue(paramType: ParameterInfo['type'], value: ParameterValue): ParameterValue {
  if (paramType === 'bool') {
    return typeof value === 'boolean' ? value : value >= 0.5;
  }

  return typeof value === 'boolean' ? (value ? 1 : 0) : value;
}

function normalizeParameter(param: ParameterInfo): ParameterInfo {
  return {
    ...param,
    value: toFrontendValue(param.type, param.value),
    default: toFrontendValue(param.type, param.default),
  };
}

export function useParameter(id: ParameterId): UseParameterResult {
  const { params, isLoading, error: sharedError, setParameter } = useAllParameters();
  const [writeError, setWriteError] = useState<Error | null>(null);

  const param = useMemo<ParameterInfo | null>(() => {
    const found = params.find((candidate) => candidate.id === id);
    if (!found) {
      return null;
    }

    return normalizeParameter(found);
  }, [id, params]);

  const notFoundError = useMemo(
    () => (!isLoading && !param ? new Error(`Parameter not found: ${id}`) : null),
    [id, isLoading, param]
  );

  const error = writeError ?? sharedError ?? notFoundError;

  const setValue = useCallback(
    async (value: ParameterValue) => {
      try {
        await setParameter(id, toBackendValue(value));
        setWriteError(null);
      } catch (err) {
        const writeErr = toError(err);
        setWriteError(writeErr);
        throw writeErr;
      }
    },
    [id, setParameter]
  );

  return { param, setValue, isLoading, error };
}
