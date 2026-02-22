import { useContext } from 'react';

import { ParameterStateContext, type ParameterStateContextValue } from './ParameterStateContext';

export function useParameterState(): ParameterStateContextValue {
  const context = useContext(ParameterStateContext);

  if (!context) {
    throw new Error('WavecraftProvider is required. Wrap your app with <WavecraftProvider>.');
  }

  return context;
}
