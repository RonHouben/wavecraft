/**
 * useParameter - Hook for managing a single parameter
 */

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from '../ipc/ParameterClient';
import { IpcBridge } from '../ipc/IpcBridge';
import { useConnectionStatus } from './useConnectionStatus';
import type { ParameterId, ParameterInfo } from '../types/parameters';

const TRANSPORT_NOT_CONNECTED = 'Transport not connected';

function toError(err: unknown): Error {
  return err instanceof Error ? err : new Error(String(err));
}

function isTransportNotConnectedError(err: unknown): boolean {
  return err instanceof Error && err.message.includes(TRANSPORT_NOT_CONNECTED);
}

export interface UseParameterResult {
  param: ParameterInfo | null;
  setValue: (value: number) => Promise<void>;
  isLoading: boolean;
  error: Error | null;
}

export function useParameter(id: ParameterId): UseParameterResult {
  const [param, setParam] = useState<ParameterInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const { connected } = useConnectionStatus();

  const loadParameter = useCallback(async () => {
    const client = ParameterClient.getInstance();
    const bridge = IpcBridge.getInstance();
    let keepLoading = false;

    try {
      setIsLoading(true);
      setError(null);

      // Get all parameters and find the one we want
      const allParams = await client.getAllParameters();
      const foundParam = allParams.find((p) => p.id === id);

      if (foundParam) {
        setParam(foundParam);
        setError(null);
      } else {
        setParam(null);
        setError(new Error(`Parameter not found: ${id}`));
      }
    } catch (err) {
      // Transient disconnect race during initial startup/reconnect.
      // Keep loading and wait for the next connection-established event.
      if (isTransportNotConnectedError(err) && !bridge.isConnected()) {
        keepLoading = true;
        setError(null);
        return;
      }

      setError(toError(err));
    } finally {
      if (!keepLoading) {
        setIsLoading(false);
      }
    }
  }, [id]);

  // Load initial parameter value
  useEffect(() => {
    if (connected) {
      loadParameter();
      return;
    }

    // Disconnected: keep stale parameter value if available, but avoid
    // rendering stale transport errors as permanent failures.
    setError((prev) => (isTransportNotConnectedError(prev) ? null : prev));
    setIsLoading(true);
  }, [connected, loadParameter]);

  // Subscribe to parameter changes
  useEffect(() => {
    const client = ParameterClient.getInstance();
    const unsubscribe = client.onParameterChanged((changedId, value) => {
      if (changedId === id) {
        setParam((prev) => (prev ? { ...prev, value } : null));
      }
    });

    return unsubscribe;
  }, [id]);

  // Set parameter value
  const setValue = useCallback(
    async (value: number) => {
      const client = ParameterClient.getInstance();
      try {
        await client.setParameter(id, value);
        // Read back authoritative host value so UI reflects clamping/remapping
        // and never diverges from backend-confirmed state.
        const confirmed = await client.getParameter(id);
        setParam((prev) => (prev ? { ...prev, value: confirmed.value } : prev));
        setError(null);
      } catch (err) {
        setError(toError(err));
        throw err;
      }
    },
    [id]
  );

  return { param, setValue, isLoading, error };
}
