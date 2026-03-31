/**
 * useSignalChainPresentation - Visual state hook for the SignalChain component
 *
 * Derives the sorted display order from the current IPC signal chain order.
 */

import type { SignalChainOrder } from '@wavecraft/core';
import { useMemo } from 'react';
import type { SignalChainEntry } from './types';

/**
 * Given the entries prop and the current unified slot order, returns an array
 * of entries sorted according to `order`.
 *
 * Unknown slot ids are skipped gracefully. If `order` is empty (still loading),
 * all entries are returned in their natural order as a loading state fallback.
 */
export function useSortedEntries(
  entries: SignalChainEntry[],
  order: SignalChainOrder[]
): SignalChainEntry[] {
  return useMemo(() => {
    // Loading fallback: show all entries in natural order
    if (order.length === 0) {
      return [...entries];
    }

    const idToEntry = new Map<string, SignalChainEntry>(entries.map((e) => [e.id, e]));

    const sorted: SignalChainEntry[] = [];

    for (const slot of order) {
      const entry = idToEntry.get(slot.id);
      if (entry) {
        sorted.push(entry);
      }
      // Unknown slot ids are silently skipped (tap/processor not registered in UI)
    }

    return sorted;
  }, [entries, order]);
}
