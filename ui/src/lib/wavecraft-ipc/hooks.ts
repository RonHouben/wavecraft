/**
 * React Hooks - High-level React integration for parameter management
 *
 * The hooks use the ParameterClient which automatically selects the correct
 * transport (WebSocket for browser dev, Native for WKWebView production).
 */

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from './ParameterClient';
import { IpcBridge } from './IpcBridge';
import type { ParameterInfo } from './types';
import { logger } from './logger/Logger';

// Lazy client initialization
let client: ParameterClient | null = null;
function getClient(): ParameterClient {
  client ??= ParameterClient.getInstance();
  return client;
}

// ============================================================================
// useParameter - Hook for managing a single parameter
// ============================================================================

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

    async function loadParameter(): Promise<void> {
      try {
        setIsLoading(true);
        setError(null);

        // Get all parameters and find the one we want
        const allParams = await getClient().getAllParameters();
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
    const unsubscribe = getClient().onParameterChanged((changedId, value) => {
      if (changedId === id) {
        setParam((prev) => (prev ? { ...prev, value } : null));
      }
    });

    return unsubscribe;
  }, [id]);

  // Set parameter value
  const setValue = useCallback(
    async (value: number) => {
      try {
        await getClient().setParameter(id, value);
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

// ============================================================================
// useAllParameters - Hook for loading all parameters
// ============================================================================

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
    try {
      setIsLoading(true);
      setError(null);
      const allParams = await getClient().getAllParameters();
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
    // Note: Nesting depth warning accepted here - inline mapper is idiomatic React pattern
    const handleParamChange = (changedId: string, value: number): void => {
      setParams((prev) => prev.map((p) => (p.id === changedId ? { ...p, value } : p)));
    };

    const unsubscribe = getClient().onParameterChanged(handleParamChange);

    return unsubscribe;
  }, []);

  return { params, isLoading, error, reload };
}

// ============================================================================
// useLatencyMonitor - Hook for monitoring IPC latency
// ============================================================================

export interface UseLatencyMonitorResult {
  latency: number | null;
  avg: number;
  max: number;
  count: number;
}

export function useLatencyMonitor(intervalMs = 1000): UseLatencyMonitorResult {
  const [latency, setLatency] = useState<number | null>(null);
  const [measurements, setMeasurements] = useState<number[]>([]);
  const bridge = IpcBridge.getInstance();

  useEffect(() => {
    let isMounted = true;

    async function measure(): Promise<void> {
      // Only measure when connected
      if (!bridge.isConnected()) {
        return;
      }

      try {
        const ms = await getClient().ping();
        if (isMounted) {
          setLatency(ms);
          setMeasurements((prev) => [...prev.slice(-99), ms]); // Keep last 100
        }
      } catch (err) {
        logger.debug('Ping failed', { error: err });
      }
    }

    // Initial measurement
    measure();

    // Periodic measurements
    const intervalId = setInterval(measure, intervalMs);

    return (): void => {
      isMounted = false;
      clearInterval(intervalId);
    };
  }, [intervalMs, bridge]);

  // Calculate statistics
  const avg =
    measurements.length > 0
      ? measurements.reduce((sum, val) => sum + val, 0) / measurements.length
      : 0;

  const max = measurements.length > 0 ? Math.max(...measurements) : 0;

  return {
    latency,
    avg,
    max,
    count: measurements.length,
  };
}
