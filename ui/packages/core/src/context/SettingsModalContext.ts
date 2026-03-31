import { createContext } from 'react';

export interface SettingsModalContextValue {
  readonly isSettingsModalOpen: boolean;
  readonly openSettingsModal: () => void;
  readonly closeSettingsModal: () => void;
}

export const SettingsModalContext = createContext<SettingsModalContextValue | undefined>(undefined);
