/**
 * useSignalChainOrder - Hook for managing the runtime signal chain slot order
 *
 * Fetches the current unified signal chain order (processors + taps) on mount,
 * subscribes to notifications, and provides an optimistic setOrder with
 * automatic rollback on error.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { SignalChainOrderClient } from '../ipc/SignalChainOrderClient';
import type { IpcError } from '../types/ipc';
import type { SignalChainOrder } from '../types/signal-chain';

export interface UseSignalChainOrderResult {
  /** Current unified signal chain slot order (processors + taps) */
  order: SignalChainOrder[];
  /** Optimistically update the order; rolls back on IPC error */
  setOrder: (slots: SignalChainOrder[]) => Promise<void>;
  isLoading: boolean;
  error: IpcError | null;
}

/**
 * @param isDraggingRef - When provided, incoming `signalChainOrderChanged`
 *   notifications are discarded while a drag is active. The optimistic order
 *   is treated as authoritative during the drag gesture.
 */
export function useSignalChainOrder(
  isDraggingRef?: React.RefObject<boolean>
): UseSignalChainOrderResult {
  const [order, setOrderState] = useState<SignalChainOrder[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<IpcError | null>(null);

  // Keep a stable ref to the latest order for use inside callbacks
  const orderRef = useRef<SignalChainOrder[]>([]);
  orderRef.current = order;

  const client = SignalChainOrderClient.getInstance();

  // Fetch initial order on mount
  useEffect(() => {
    let cancelled = false;

    setIsLoading(true);
    setError(null);

    client
      .getSignalChainOrder()
      .then((fetchedSlots) => {
        if (!cancelled) {
          setOrderState(fetchedSlots);
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

  // Subscribe to signalChainOrderChanged notifications
  useEffect(() => {
    const unsubscribe = client.onSignalChainOrderChanged((newSlots) => {
      // Discard incoming notifications while a drag is in progress
      if (isDraggingRef?.current) {
        return;
      }
      setOrderState(newSlots);
      setError(null);
    });

    return unsubscribe;
    // isDraggingRef is a stable ref — no need in deps
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setOrder = useCallback(
    async (newSlots: SignalChainOrder[]) => {
      const previousOrder = orderRef.current;

      // Optimistic update
      setOrderState(newSlots);
      setError(null);

      try {
        await client.setSignalChainOrder(newSlots);
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
