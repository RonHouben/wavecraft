import { useSyncExternalStore } from 'react';

import {
  getRegisteredProcessorsSnapshot,
  subscribeToRegisteredProcessors,
} from '../processors/registry';

/**
 * Returns the currently registered processor IDs.
 */
export function useAvailableProcessors(): readonly string[] {
  return useSyncExternalStore(
    subscribeToRegisteredProcessors,
    getRegisteredProcessorsSnapshot,
    getRegisteredProcessorsSnapshot
  );
}
