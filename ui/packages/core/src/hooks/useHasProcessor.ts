import { useSyncExternalStore } from 'react';
import type { ProcessorId } from '../types/processors';

import {
  getRegisteredProcessorsSnapshot,
  subscribeToRegisteredProcessors,
} from '../processors/registry';

/**
 * Returns true when a processor ID is present in the generated processor registry.
 */
export function useHasProcessorInSignalChain(processorId: ProcessorId): boolean {
  const availableProcessors = useSyncExternalStore(
    subscribeToRegisteredProcessors,
    getRegisteredProcessorsSnapshot,
    getRegisteredProcessorsSnapshot
  );

  return availableProcessors.includes(processorId);
}
