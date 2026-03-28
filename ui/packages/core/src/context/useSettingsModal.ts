import { useContext } from 'react';

import { SettingsModalContext, type SettingsModalContextValue } from './SettingsModalContext';

export function useSettingsModal(): SettingsModalContextValue {
  const context = useContext(SettingsModalContext);

  if (!context) {
    throw new Error('WavecraftProvider is required. Wrap your app with <WavecraftProvider>.');
  }

  return context;
}
