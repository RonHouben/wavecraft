/**
 * useProcessorOrder - Hook for managing the runtime processor signal-chain order
 *
 * Fetches the current processor order on mount, subscribes to notifications,
 * and provides an optimistic setOrder with automatic rollback on error.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { ProcessorOrderClient } from '../ipc/ProcessorOrderClient';
import type { IpcError } from '../types/ipc';

export interface UseProcessorOrderResult {
  /** Current processor order (slot indices as strings, e.g. ["0", "1", "2"]) */
  order: string[];
  /** Optimistically update the order; rolls back on IPC error */
  setOrder: (order: string[]) => Promise<void>;
  isLoading: boolean;
  error: IpcError | null;
}

/**
 * @param isDraggingRef - When provided, incoming `processorOrderChanged`
 *   notifications are discarded while a drag is active. The optimistic order
 *   is treated as authoritative during the drag gesture.
 */
export function useProcessorOrder(
  isDraggingRef?: React.RefObject<boolean>
): UseProcessorOrderResult {
  const [order, setOrderState] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<IpcError | null>(null);

  // Keep a stable ref to the latest order for use inside callbacks
  const orderRef = useRef<string[]>([]);
  orderRef.current = order;

  const client = ProcessorOrderClient.getInstance();

  // Fetch initial order on mount
  useEffect(() => {
    let cancelled = false;

    setIsLoading(true);
    setError(null);

    client
      .getProcessorOrder()
      .then((fetchedOrder) => {
        if (!cancelled) {
          setOrderState(fetchedOrder);
          setIsLoading(false);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const ipcError: IpcError = toIpcError(err);
          setError(ipcError);
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Subscribe to processorOrderChanged notifications
  useEffect(() => {
    const unsubscribe = client.onProcessorOrderChanged((newOrder) => {
      // Discard incoming notifications while a drag is in progress
      if (isDraggingRef?.current) {
        return;
      }
      setOrderState(newOrder);
      setError(null);
    });

    return unsubscribe;
    // isDraggingRef is a stable ref — no need in deps
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setOrder = useCallback(
    async (newOrder: string[]) => {
      const previousOrder = orderRef.current;

      // Optimistic update
      setOrderState(newOrder);
      setError(null);

      try {
        await client.setProcessorOrder(newOrder);
      } catch (err: unknown) {
        // Rollback on error
        setOrderState(previousOrder);
        setError(toIpcError(err));
        throw err;
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );

  return { order, setOrder, isLoading, error };
}

function toIpcError(err: unknown): IpcError {
  if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
    return err as IpcError;
  }
  return {
    code: -32000,
    message: err instanceof Error ? err.message : String(err),
  };
}
