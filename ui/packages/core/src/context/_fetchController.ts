import type { Dispatch, MutableRefObject, SetStateAction } from 'react';

import { IpcBridge } from '../ipc/IpcBridge';
import { logger } from '../logger/Logger';
import { ParameterClient } from '../ipc/ParameterClient';
import type { ParameterInfo } from '../types/parameters';

import { normalizeParameter } from './_valueHelpers';

// private — do not export from index.ts
export interface FetchAttemptResult {
  success: boolean;
  shouldStop: boolean;
  params?: ParameterInfo[];
  error?: Error;
}

// private — do not export from index.ts
export async function attemptFetch(
  client: ParameterClient,
  attempt: number,
  maxFetchRetries: number
): Promise<FetchAttemptResult> {
  try {
    if (attempt > 0) {
      logger.debug('WavecraftProvider: retry attempt', {
        attempt,
        maxRetries: maxFetchRetries,
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

    if (attempt === maxFetchRetries) {
      const message = err instanceof Error ? err.message : String(err);
      const error = new Error(
        `Parameter fetch failed after ${maxFetchRetries + 1} attempts: ${message}`
      );
      return { success: false, shouldStop: true, error };
    }

    return { success: false, shouldStop: false };
  }
}

// private — do not export from index.ts
export function handleSuccessResult(
  result: FetchAttemptResult,
  mountedRef: MutableRefObject<boolean>,
  setParams: Dispatch<SetStateAction<ParameterInfo[]>>,
  setError: Dispatch<SetStateAction<Error | null>>,
  setIsLoading: Dispatch<SetStateAction<boolean>>,
  fetchingRef: MutableRefObject<boolean>
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

// private — do not export from index.ts
export function handleStopResult(
  result: FetchAttemptResult,
  mountedRef: MutableRefObject<boolean>,
  setError: Dispatch<SetStateAction<Error | null>>,
  setIsLoading: Dispatch<SetStateAction<boolean>>,
  fetchingRef: MutableRefObject<boolean>
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
