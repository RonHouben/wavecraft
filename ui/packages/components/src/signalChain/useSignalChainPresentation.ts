/**
 * useSignalChainPresentation - Visual state hook for the SignalChain component
 *
 * Derives the sorted display order from the current IPC order.
 */

import { useMemo } from 'react';
import type { SignalChainProcessorEntry } from './types';

/**
 * Given the processors prop and the current slot-index order, returns an array
 * of processor entries sorted according to `order`.
 *
 * Only entries whose slot index appears in `order` are included. If `order` is
 * empty (still loading), all processors are returned in their natural order as
 * a loading state fallback.
 */
export function useSortedProcessors(
  processors: SignalChainProcessorEntry[],
  order: string[]
): SignalChainProcessorEntry[] {
  return useMemo(() => {
    // Loading fallback: show all processors in natural order
    if (order.length === 0) {
      return [...processors];
    }

    const slotToEntry = new Map<string, SignalChainProcessorEntry>(
      processors.map((p, i) => [String(i), p])
    );

    const sorted: SignalChainProcessorEntry[] = [];

    for (const slotIdx of order) {
      const entry = slotToEntry.get(slotIdx);
      if (entry) {
        sorted.push(entry);
      }
    }

    return sorted;
  }, [processors, order]);
}
