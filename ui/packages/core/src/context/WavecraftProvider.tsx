import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

import { useConnectionStatus } from '../hooks/useConnectionStatus';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import { logger } from '../logger/Logger';
import type { ParameterId, ParameterInfo, ParameterValue } from '../types/parameters';
import { ParameterStateContext, type ParameterStateContextValue } from './ParameterStateContext';
import { SettingsModalContext, type SettingsModalContextValue } from './SettingsModalContext';
import { attemptFetch, handleStopResult, handleSuccessResult } from './_fetchController';
import {
  wireParameterChangedSubscription,
  wireParametersChangedReload,
} from './_subscriptionWiring';
import { updateParameterValue } from './_valueHelpers';
import { createSetParameterHandler } from './_writeReconciler';

/** Maximum time (ms) to wait for connection before giving up */
const CONNECTION_TIMEOUT_MS = 15_000;

/** Maximum fetch retry attempts after connection is established */
const MAX_FETCH_RETRIES = 3;

/** Base delay (ms) for fetch retry backoff */
const FETCH_RETRY_BASE_MS = 500;

type WavecraftProviderRenderProps = {
  openSettingsModal: SettingsModalContextValue['openSettingsModal'];
};
export interface WavecraftProviderProps {
  children: ReactNode | ((ctx: WavecraftProviderRenderProps) => ReactNode);
}

export function WavecraftProvider(props: Readonly<WavecraftProviderProps>) {
  const [params, setParams] = useState<ParameterInfo<ParameterValue>[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);

  const { connected } = useConnectionStatus();

  const mountedRef = useRef(true);
  const paramsRef = useRef<ParameterInfo<ParameterValue>[]>([]);
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

      const result = await attemptFetch(client, attempt, MAX_FETCH_RETRIES);

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
      const setParameterHandler = createSetParameterHandler(paramsRef, setParams, setError);
      return setParameterHandler(id, value);
    },
    []
  );

  const openSettingsModal = useCallback((): void => {
    setIsSettingsModalOpen(true);
  }, []);

  const closeSettingsModal = useCallback((): void => {
    setIsSettingsModalOpen(false);
  }, []);

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
    const handleParameterChanged = (changedId: ParameterId, value: ParameterValue): void => {
      setParams((prev) => updateParameterValue(prev, changedId, value));
    };

    return wireParameterChangedSubscription(handleParameterChanged);
  }, []);

  useEffect(() => {
    return wireParametersChangedReload(reload);
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

  const settingsModalValue = useMemo<SettingsModalContextValue>(
    () => ({
      isSettingsModalOpen,
      openSettingsModal,
      closeSettingsModal,
    }),
    [closeSettingsModal, isSettingsModalOpen, openSettingsModal]
  );

  return (
    <ParameterStateContext.Provider value={value}>
      <SettingsModalContext.Provider value={settingsModalValue}>
        {renderChildren(props.children, { openSettingsModal })}
      </SettingsModalContext.Provider>
    </ParameterStateContext.Provider>
  );
}

function renderChildren(
  children: WavecraftProviderProps['children'],
  renderProps: WavecraftProviderRenderProps
) {
  if (typeof children === 'function') {
    return children(renderProps);
  }

  return children;
}
