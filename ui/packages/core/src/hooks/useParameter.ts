/**
 * useParameter - Hook for managing a single parameter
 */

import { useCallback, useMemo, useState } from 'react';

import { useAllParameters } from './useAllParameters';
import type {
  ParameterId,
  ParameterInfo,
  ParameterValue,
  ParameterVariant,
} from '../types/parameters';

function toError(err: unknown): Error {
  return err instanceof Error ? err : new Error(String(err));
}

export interface UseParameterResult<T extends ParameterValue, V extends ParameterVariant> {
  param: ParameterInfo<T, V> | null;
  setValue: (value: T) => Promise<void>;
  isLoading: boolean;
  error: Error | null;
}

function toBackendValue(value: ParameterValue): number {
  return typeof value === 'boolean' ? (value ? 1 : 0) : typeof value === 'number' ? value : 0;
}

function toFrontendValue<T extends ParameterValue, V extends ParameterVariant>(
  paramType: ParameterInfo<T, V>['type'],
  value: T
): T {
  if (paramType === 'bool') {
    return (typeof value === 'boolean' ? value : (value as number) >= 0.5) as T;
  }

  return (typeof value === 'boolean' ? (value ? 1 : 0) : value) as T;
}

function normalizeParameter<T extends ParameterValue, V extends ParameterVariant>(
  param: ParameterInfo<T, V>
): ParameterInfo<T, V> {
  return {
    ...param,
    value: toFrontendValue(param.type, param.value),
    default: toFrontendValue(param.type, param.default),
  };
}

export function useParameter<T extends ParameterValue, V extends ParameterVariant = undefined>(
  id: ParameterId
): UseParameterResult<T, V> {
  const { params, isLoading, error: sharedError, setParameter } = useAllParameters();
  const [writeError, setWriteError] = useState<Error | null>(null);

  const param = useMemo<ParameterInfo<T, V> | null>(() => {
    const found = params.find((candidate) => candidate.id === id);
    if (!found) {
      return null;
    }

    return normalizeParameter<T, V>(found);
  }, [id, params]);

  const notFoundError = useMemo(
    () => (!isLoading && !param ? new Error(`Parameter not found: ${id}`) : null),
    [id, isLoading, param]
  );

  const error = writeError ?? sharedError ?? notFoundError;

  const setValue = useCallback(
    async (value: T) => {
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
