import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

import { useConnectionStatus } from '../hooks/useConnectionStatus';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcEvents } from '../ipc/constants';
import { ParameterClient } from '../ipc/ParameterClient';
import { logger } from '../logger/Logger';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';
import { ParameterStateContext, type ParameterStateContextValue } from './ParameterStateContext';

/** Maximum time (ms) to wait for connection before giving up */
const CONNECTION_TIMEOUT_MS = 15_000;

/** Maximum fetch retry attempts after connection is established */
const MAX_FETCH_RETRIES = 3;

/** Base delay (ms) for fetch retry backoff */
const FETCH_RETRY_BASE_MS = 500;

interface FetchAttemptResult {
  success: boolean;
  shouldStop: boolean;
  params?: ParameterInfo[];
  error?: Error;
}

export interface WavecraftProviderProps {
  children: ReactNode;
}

function toError(err: unknown): Error {
  return err instanceof Error ? err : new Error(String(err));
}

function normalizeValue(paramType: ParameterInfo['type'], value: ParameterValue): ParameterValue {
  if (paramType === 'bool') {
    return typeof value === 'boolean' ? value : value >= 0.5;
  }

  return typeof value === 'boolean' ? (value ? 1 : 0) : value;
}

function normalizeParameter(param: ParameterInfo): ParameterInfo {
  return {
    ...param,
    value: normalizeValue(param.type, param.value),
    default: normalizeValue(param.type, param.default),
  };
}

function updateParameterValue(
  params: ParameterInfo[],
  changedId: ParameterId,
  value: ParameterValue
): ParameterInfo[] {
  let changed = false;

  const next = params.map((param) => {
    if (param.id !== changedId) {
      return param;
    }

    const normalizedValue = normalizeValue(param.type, value);
    if (param.value === normalizedValue) {
      return param;
    }

    changed = true;
    return {
      ...param,
      value: normalizedValue,
    };
  });

  return changed ? next : params;
}

function rollbackParameterValueIfCurrentMatches(
  params: ParameterInfo[],
  changedId: ParameterId,
  expectedCurrentValue: ParameterValue,
  rollbackValue: ParameterValue
): ParameterInfo[] {
  let changed = false;

  const next = params.map((param) => {
    if (param.id !== changedId) {
      return param;
    }

    const normalizedExpected = normalizeValue(param.type, expectedCurrentValue);
    if (param.value !== normalizedExpected) {
      return param;
    }

    const normalizedRollback = normalizeValue(param.type, rollbackValue);
    if (param.value === normalizedRollback) {
      return param;
    }

    changed = true;
    return {
      ...param,
      value: normalizedRollback,
    };
  });

  return changed ? next : params;
}

async function attemptFetch(client: ParameterClient, attempt: number): Promise<FetchAttemptResult> {
  try {
    if (attempt > 0) {
      logger.debug('WavecraftProvider: retry attempt', {
        attempt,
        maxRetries: MAX_FETCH_RETRIES,
      });
    }

    const allParams = await client.getAllParameters();
    return { success: true, shouldStop: false, params: allParams.map(normalizeParameter) };
  } catch (err) {
    const bridge = IpcBridge.getInstance();
    if (!bridge.isConnected()) {
      logger.debug('WavecraftProvider: transport disconnected during fetch, awaiting reconnect');
      return { success: false, shouldStop: true };
    }

    if (attempt === MAX_FETCH_RETRIES) {
      const message = err instanceof Error ? err.message : String(err);
      const error = new Error(
        `Parameter fetch failed after ${MAX_FETCH_RETRIES + 1} attempts: ${message}`
      );
      return { success: false, shouldStop: true, error };
    }

    return { success: false, shouldStop: false };
  }
}

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

export function WavecraftProvider({ children }: Readonly<WavecraftProviderProps>) {
  const [params, setParams] = useState<ParameterInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const { connected } = useConnectionStatus();

  const mountedRef = useRef(true);
  const paramsRef = useRef<ParameterInfo[]>([]);
  const fetchingRef = useRef(false);
  const prevConnectedRef = useRef<boolean | null>(null);

  useEffect(() => {
    paramsRef.current = params;
  }, [params]);

  useEffect(() => {
    mountedRef.current = true;

    return () => {
      mountedRef.current = false;
    };
  }, []);

  const fetchParameters = useCallback(async (): Promise<void> => {
    if (fetchingRef.current) {
      logger.debug('WavecraftProvider: fetch already in-flight, skipping');
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

      if (handleSuccessResult(result, mountedRef, setParams, setError, setIsLoading, fetchingRef)) {
        return;
      }

      if (handleStopResult(result, mountedRef, setError, setIsLoading, fetchingRef)) {
        return;
      }

      const delay = FETCH_RETRY_BASE_MS * Math.pow(2, attempt);
      await new Promise((resolve) => setTimeout(resolve, delay));
    }

    fetchingRef.current = false;
  }, []);

  const reload = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) {
      return;
    }

    setIsLoading(true);
    setError(null);

    const bridge = IpcBridge.getInstance();
    if (bridge.isConnected()) {
      await fetchParameters();
    }
  }, [fetchParameters]);

  const setParameter = useCallback(
    async (id: ParameterId, value: ParameterValue): Promise<void> => {
      const client = ParameterClient.getInstance();
      const target = paramsRef.current.find((param) => param.id === id);
      const previousValue = target?.value;
      const optimisticValue = target ? normalizeValue(target.type, value) : value;

      if (target && target.value !== optimisticValue) {
        setParams((prev) => updateParameterValue(prev, id, optimisticValue));
      }

      try {
        await client.setParameter(id, value);
        setError(null);
      } catch (err) {
        if (previousValue !== undefined) {
          setParams((prev) =>
            rollbackParameterValueIfCurrentMatches(
              prev,
              id,
              optimisticValue,
              previousValue as ParameterValue
            )
          );
        }

        const writeError = toError(err);
        setError(writeError);
        throw writeError;
      }
    },
    []
  );

  useEffect(() => {
    const wasConnected = prevConnectedRef.current;
    prevConnectedRef.current = connected;

    if (connected && wasConnected !== true) {
      logger.debug('WavecraftProvider: connection established, fetching parameters');
      setError(null);
      void fetchParameters();
    }

    if (!connected && wasConnected === true) {
      logger.debug('WavecraftProvider: connection lost, awaiting reconnect');
      setIsLoading(true);
    }
  }, [connected, fetchParameters]);

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!mountedRef.current) {
        return;
      }

      const bridge = IpcBridge.getInstance();
      if (!bridge.isConnected()) {
        setError(
          new Error(
            'Could not connect to dev server within 15 seconds. Is `wavecraft start` running?'
          )
        );
        setIsLoading(false);
      }
    }, CONNECTION_TIMEOUT_MS);

    return () => clearTimeout(timeoutId);
  }, []);

  useEffect(() => {
    const client = ParameterClient.getInstance();
    const handleParameterChanged = (changedId: ParameterId, value: ParameterValue): void => {
      setParams((prev) => updateParameterValue(prev, changedId, value));
    };

    return client.onParameterChanged(handleParameterChanged);
  }, []);

  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const unsubscribe = bridge.on(IpcEvents.PARAMETERS_CHANGED, () => {
      logger.info('Parameters changed on server (hot-reload), re-fetching...');
      void reload();
    });

    return unsubscribe;
  }, [reload]);

  const value = useMemo<ParameterStateContextValue>(
    () => ({
      params,
      isLoading,
      error,
      setParameter,
      reload,
    }),
    [error, isLoading, params, reload, setParameter]
  );

  return <ParameterStateContext.Provider value={value}>{children}</ParameterStateContext.Provider>;
}
