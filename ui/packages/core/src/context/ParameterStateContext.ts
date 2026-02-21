import { createContext } from 'react';

import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';

export interface ParameterStateContextValue {
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  setParameter: (id: ParameterId, value: ParameterValue) => Promise<void>;
  reload: () => Promise<void>;
}

export const ParameterStateContext = createContext<ParameterStateContextValue | undefined>(
  undefined
);
