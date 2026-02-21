import type { Dispatch, MutableRefObject, SetStateAction } from 'react';

import { ParameterClient } from '../ipc/ParameterClient';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';

import {
  normalizeValue,
  rollbackParameterValueIfCurrentMatches,
  toError,
  updateParameterValue,
} from './_valueHelpers';

// private — do not export from index.ts
export function createSetParameterHandler(
  paramsRef: MutableRefObject<ParameterInfo[]>,
  setParams: Dispatch<SetStateAction<ParameterInfo[]>>,
  setError: Dispatch<SetStateAction<Error | null>>
): (id: ParameterId, value: ParameterValue) => Promise<void> {
  return async (id: ParameterId, value: ParameterValue): Promise<void> => {
    const client = ParameterClient.getInstance();
    const target = paramsRef.current.find((param) => param.id === id);
    const previousValue = target?.value;
    const optimisticValue = target ? normalizeValue(target.type, value) : value;

    if (target && target.value !== optimisticValue) {
      setParams((prev) => updateParameterValue(prev, id, optimisticValue));
    }

    try {
      await client.setParameter(id, value);
      setError(null);
    } catch (err) {
      if (previousValue !== undefined) {
        setParams((prev) =>
          rollbackParameterValueIfCurrentMatches(
            prev,
            id,
            optimisticValue,
            previousValue as ParameterValue
          )
        );
      }

      const writeError = toError(err);
      setError(writeError);
      throw writeError;
    }
  };
}
