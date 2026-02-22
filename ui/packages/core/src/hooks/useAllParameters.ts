import { useParameterState } from '../context/useParameterState';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';

export interface UseAllParametersResult {
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  setParameter: (id: ParameterId, value: ParameterValue) => Promise<void>;
  reload: () => Promise<void>;
}

export function useAllParameters(): UseAllParametersResult {
  return useParameterState();
}
