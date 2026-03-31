/**
 * useSignalChainOrder - Hook for managing the runtime signal chain slot order
 *
 * Fetches the current unified signal chain order (processors + taps) on mount,
 * subscribes to notifications, and provides an optimistic setOrder with
 * automatic rollback on error.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { IpcBridge } from '../ipc/IpcBridge';
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
  const [order, setOrder] = useState<SignalChainOrder[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<IpcError | null>(null);

  // Keep a stable ref to the latest order for use inside callbacks
  const orderRef = useRef<SignalChainOrder[]>([]);
  orderRef.current = order;

  const client = SignalChainOrderClient.getInstance();
  const bridge = IpcBridge.getInstance();

  useEffect(() => {
    let cancelled = false;

    const fetchOrder = async (): Promise<void> => {
      if (!bridge.isConnected()) {
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const fetchedSlots = await client.getSignalChainOrder();
        if (!cancelled) {
          setOrder(fetchedSlots);
          setIsLoading(false);
        }
      } catch (err: unknown) {
        if (!cancelled && bridge.isConnected()) {
          const ipcError: IpcError = toIpcError(err);
          setError(ipcError);
          setIsLoading(false);
        }
      }
    };

    const unsubscribeConnection = bridge.onConnectionChange((connected) => {
      if (!connected) {
        if (!cancelled) {
          setError(null);
          setIsLoading(orderRef.current.length === 0);
        }
        return;
      }

      void fetchOrder();
    });

    void fetchOrder();

    return () => {
      cancelled = true;
      unsubscribeConnection();
    };
  }, [bridge, client]);

  // Subscribe to signalChainOrderChanged notifications
  useEffect(() => {
    const unsubscribe = client.onSignalChainOrderChanged((newSlots) => {
      // Discard incoming notifications while a drag is in progress
      if (isDraggingRef?.current) {
        return;
      }
      setOrder(newSlots);
      setError(null);
      setIsLoading(false);
    });

    return unsubscribe;
    // isDraggingRef is a stable ref — no need in deps
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const updateOrder = useCallback(
    async (newSlots: SignalChainOrder[]) => {
      const previousOrder = orderRef.current;

      // Optimistic update
      setOrder(newSlots);
      setError(null);
      setIsLoading(false);

      try {
        await client.setSignalChainOrder(newSlots);
      } catch (err: unknown) {
        // Rollback on error
        setOrder(previousOrder);
        setError(toIpcError(err));
        setIsLoading(false);
        throw err;
      }
    },
    [client]
  );

  return { order, setOrder: updateOrder, isLoading, error };
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
