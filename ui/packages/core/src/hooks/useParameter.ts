/**
 * useParameter - Hook for managing a single parameter
 */

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from '../ipc/ParameterClient';
import type { ParameterInfo } from '../types/parameters';

export interface UseParameterResult {
  param: ParameterInfo | null;
  setValue: (value: number) => Promise<void>;
  isLoading: boolean;
  error: Error | null;
}

export function useParameter(id: string): UseParameterResult {
  const [param, setParam] = useState<ParameterInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  // Load initial parameter value
  useEffect(() => {
    let isMounted = true;
    const client = ParameterClient.getInstance();

    async function loadParameter(): Promise<void> {
      try {
        setIsLoading(true);
        setError(null);

        // Get all parameters and find the one we want
        const allParams = await client.getAllParameters();
        const foundParam = allParams.find((p) => p.id === id);

        if (isMounted) {
          if (foundParam) {
            setParam(foundParam);
          } else {
            setError(new Error(`Parameter not found: ${id}`));
          }
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)));
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    loadParameter();

    return (): void => {
      isMounted = false;
    };
  }, [id]);

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
        // Optimistically update local state
        setParam((prev) => (prev ? { ...prev, value } : null));
      } catch (err) {
        setError(err instanceof Error ? err : new Error(String(err)));
        throw err;
      }
    },
    [id]
  );

  return { param, setValue, isLoading, error };
}
