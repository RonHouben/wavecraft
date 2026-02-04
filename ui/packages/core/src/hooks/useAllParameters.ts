/**
 * useAllParameters - Hook for loading all parameters
 */

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from '../ipc/ParameterClient';
import type { ParameterInfo } from '../types/parameters';

export interface UseAllParametersResult {
  params: ParameterInfo[];
  isLoading: boolean;
  error: Error | null;
  reload: () => Promise<void>;
}

export function useAllParameters(): UseAllParametersResult {
  const [params, setParams] = useState<ParameterInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const reload = useCallback(async () => {
    const client = ParameterClient.getInstance();
    try {
      setIsLoading(true);
      setError(null);
      const allParams = await client.getAllParameters();
      setParams(allParams);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load on mount
  useEffect(() => {
    reload();
  }, [reload]);

  // Subscribe to parameter changes
  useEffect(() => {
    const client = ParameterClient.getInstance();
    // Note: Nesting depth warning accepted here - inline mapper is idiomatic React pattern
    const handleParamChange = (changedId: string, value: number): void => {
      setParams((prev) => prev.map((p) => (p.id === changedId ? { ...p, value } : p)));
    };

    const unsubscribe = client.onParameterChanged(handleParamChange);

    return unsubscribe;
  }, []);

  return { params, isLoading, error, reload };
}
