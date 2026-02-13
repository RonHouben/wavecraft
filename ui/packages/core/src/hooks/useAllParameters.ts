/**
 * useAllParameters - Hook for loading all parameters
 *
 * Connection-aware: waits for transport connection before fetching.
 * Auto-refetches on reconnection. Deduplicates concurrent requests.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { ParameterClient } from '../ipc/ParameterClient';
import { IpcBridge } from '../ipc/IpcBridge';
import { useConnectionStatus } from './useConnectionStatus';
import { logger } from '../logger/Logger';
import type { ParameterInfo } from '../types/parameters';

/** Maximum time (ms) to wait for connection before giving up */
const CONNECTION_TIMEOUT_MS = 15_000;

/** Maximum fetch retry attempts after connection is established */
const MAX_FETCH_RETRIES = 3;

/** Base delay (ms) for fetch retry backoff */
const FETCH_RETRY_BASE_MS = 500;

/**
 * Result of a single fetch attempt
 */
interface FetchAttemptResult {
  success: boolean;
  shouldStop: boolean;
  params?: ParameterInfo[];
  error?: Error;
}

/**
 * Attempt to fetch parameters (helper to reduce complexity)
 */
async function attemptFetch(client: ParameterClient, attempt: number): Promise<FetchAttemptResult> {
  try {
    if (attempt > 0) {
      logger.debug('useAllParameters: retry attempt', {
        attempt,
        maxRetries: MAX_FETCH_RETRIES,
      });
    }

    const allParams = await client.getAllParameters();
    return { success: true, shouldStop: false, params: allParams };
  } catch (err) {
    // If transport disconnected mid-fetch, don't retry
    const bridge = IpcBridge.getInstance();
    if (!bridge.isConnected()) {
      logger.debug('useAllParameters: transport disconnected during fetch, awaiting reconnect');
      return { success: false, shouldStop: true };
    }

    // Last retry exhausted → surface error
    if (attempt === MAX_FETCH_RETRIES) {
      const message = err instanceof Error ? err.message : String(err);
      const error = new Error(
        `Parameter fetch failed after ${MAX_FETCH_RETRIES + 1} attempts: ${message}`
      );
      return { success: false, shouldStop: true, error };
    }

    // Continue retrying
    return { success: false, shouldStop: false };
  }
}

/**
 * Update a single parameter in the list
 */
function updateParameterValue(
  params: ParameterInfo[],
  changedId: string,
  value: number
): ParameterInfo[] {
  return params.map((p) => (p.id === changedId ? { ...p, value } : p));
}

/**
 * Handle successful fetch result
 */
function handleSuccessResult(
  result: FetchAttemptResult,
  mountedRef: React.MutableRefObject<boolean>,
  setParams: React.Dispatch<React.SetStateAction<ParameterInfo[]>>,
  setError: React.Dispatch<React.SetStateAction<Error | null>>,
  setIsLoading: React.Dispatch<React.SetStateAction<boolean>>,
  fetchingRef: React.MutableRefObject<boolean>
): boolean {
  if (result.success && mountedRef.current && result.params) {
    setParams(result.params);
    setError(null);
    setIsLoading(false);
    fetchingRef.current = false;
    return true;
  }
  return false;
}

/**
 * Handle stop condition (error or disconnect)
 */
function handleStopResult(
  result: FetchAttemptResult,
  mountedRef: React.MutableRefObject<boolean>,
  setError: React.Dispatch<React.SetStateAction<Error | null>>,
  setIsLoading: React.Dispatch<React.SetStateAction<boolean>>,
  fetchingRef: React.MutableRefObject<boolean>
): boolean {
  if (result.shouldStop) {
    if (mountedRef.current && result.error) {
      setError(result.error);
      setIsLoading(false);
    }
    fetchingRef.current = false;
    return true;
  }
  return false;
}

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

  const { connected } = useConnectionStatus();

  // Cleanup guard: prevents setState on unmounted component
  const mountedRef = useRef(true);
  // Deduplication: prevents concurrent fetches
  const fetchingRef = useRef(false);
  // Track previous connected state to detect transitions
  const prevConnectedRef = useRef<boolean | null>(null);

  // Mount/unmount lifecycle
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  /**
   * Fetch all parameters with retry logic.
   * Only retries on application-level failures while transport is connected.
   * Bails out silently if transport disconnects mid-fetch (reconnect handler
   * will re-trigger).
   */
  const fetchParameters = useCallback(async (): Promise<void> => {
    if (fetchingRef.current) {
      logger.debug('useAllParameters: fetch already in-flight, skipping');
      return;
    }

    fetchingRef.current = true;
    const client = ParameterClient.getInstance();

    for (let attempt = 0; attempt <= MAX_FETCH_RETRIES; attempt++) {
      if (!mountedRef.current) {
        fetchingRef.current = false;
        return;
      }

      const result = await attemptFetch(client, attempt);

      // Handle success
      if (handleSuccessResult(result, mountedRef, setParams, setError, setIsLoading, fetchingRef)) {
        return;
      }

      // Handle stop condition (error or disconnect)
      if (handleStopResult(result, mountedRef, setError, setIsLoading, fetchingRef)) {
        return;
      }

      // Continue retrying: exponential backoff
      const delay = FETCH_RETRY_BASE_MS * Math.pow(2, attempt);
      await new Promise((resolve) => setTimeout(resolve, delay));
    }

    fetchingRef.current = false;
  }, []);

  /**
   * Public reload function.
   * Connection-aware: if connected → fetch immediately.
   * If disconnected → resets to loading state (connection effect handles retry).
   */
  const reload = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    setIsLoading(true);
    setError(null);

    const bridge = IpcBridge.getInstance();
    if (bridge.isConnected()) {
      await fetchParameters();
    }
    // If not connected: isLoading stays true.
    // The connection change effect will trigger fetchParameters on connect.
  }, [fetchParameters]);

  // ─── Effect: React to connection state transitions ──────────────────
  useEffect(() => {
    const wasConnected = prevConnectedRef.current;
    prevConnectedRef.current = connected;

    if (connected && wasConnected !== true) {
      // Transition: disconnected/initial → connected
      logger.debug('useAllParameters: connection established, fetching parameters');
      // Clear any timeout error when we successfully connect
      setError(null);
      fetchParameters();
    }

    if (!connected && wasConnected === true) {
      // Transition: connected → disconnected
      // Keep stale params (better than empty). Show loading for incoming refetch.
      logger.debug('useAllParameters: connection lost, awaiting reconnect');
      setIsLoading(true);
    }
  }, [connected, fetchParameters]);

  // ─── Effect: Connection timeout ─────────────────────────────────────
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!mountedRef.current) return;

      // Only set error if we're still not connected after timeout
      const bridge = IpcBridge.getInstance();
      if (!bridge.isConnected()) {
        setError(
          new Error(
            'Could not connect to dev server within 15 seconds. ' + 'Is `wavecraft start` running?'
          )
        );
        setIsLoading(false);
      }
    }, CONNECTION_TIMEOUT_MS);

    return () => clearTimeout(timeoutId);
  }, []); // Empty deps - timeout fires once after mount

  // ─── Effect: Subscribe to parameter change notifications ────────────
  useEffect(() => {
    const client = ParameterClient.getInstance();
    const handleParamChange = (changedId: string, value: number): void => {
      setParams((prev) => updateParameterValue(prev, changedId, value));
    };
    return client.onParameterChanged(handleParamChange);
  }, []);

  // ─── Effect: Subscribe to hot-reload notifications ─────────────────
  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const unsubscribe = bridge.on('parametersChanged', () => {
      logger.info('Parameters changed on server (hot-reload), re-fetching...');
      reload();
    });
    return unsubscribe;
  }, [reload]);

  return { params, isLoading, error, reload };
}
