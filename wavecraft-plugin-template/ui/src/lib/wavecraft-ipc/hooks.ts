/**
 * React Hooks - High-level React integration for parameter management
 *
 * Environment-aware: In browser mode (development), these hooks return
 * mock data without attempting IPC. In WKWebView mode (production), they
 * use real IPC.
 */

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from './ParameterClient';
import { IpcBridge } from './IpcBridge';
import type { ParameterInfo } from './types';
import { isBrowserEnvironment } from './environment';
import { logger } from './logger/Logger';

// Detect environment once at module load time.
// Must be evaluated at module scope (not inside hooks) to comply with React's
// Rules of Hooks - conditional hook calls based on runtime checks inside hook
// bodies would violate hook call order consistency.
const IS_BROWSER = isBrowserEnvironment();

// Lazy client initialization - only create if in WebView mode
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
  // Default mock data for browser mode
  const mockParam: ParameterInfo = {
    id,
    name: id.charAt(0).toUpperCase() + id.slice(1),
    value: 0.5,
    default: 0.5,
    type: 'float',
  };

  const [param, setParam] = useState<ParameterInfo | null>(IS_BROWSER ? mockParam : null);
  const [isLoading, setIsLoading] = useState(!IS_BROWSER);
  const [error, setError] = useState<Error | null>(null);

  // Load initial parameter value (skip in browser mode)
  useEffect(() => {
    if (IS_BROWSER) return;

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

  // Subscribe to parameter changes (skip in browser mode)
  useEffect(() => {
    if (IS_BROWSER) return;

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
      if (IS_BROWSER) {
        // In browser mode, just update local state
        setParam((prev) => (prev ? { ...prev, value } : null));
        return;
      }

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
  const [isLoading, setIsLoading] = useState(!IS_BROWSER);
  const [error, setError] = useState<Error | null>(null);

  const reload = useCallback(async () => {
    if (IS_BROWSER) {
      // In browser mode, no-op
      return;
    }

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

  // Load on mount (skip in browser mode)
  useEffect(() => {
    if (IS_BROWSER) return;
    reload();
  }, [reload]);

  // Subscribe to parameter changes (skip in browser mode)
  useEffect(() => {
    if (IS_BROWSER) return;

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
    if (IS_BROWSER) {
      // In browser mode, return mock data
      return;
    }

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
        logger.error('LatencyMonitor ping failed', { error: err });
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
