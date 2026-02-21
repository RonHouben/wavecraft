/**
 * useAudioStatus - Monitor runtime audio readiness independently from transport connection.
 *
 * `useConnectionStatus()` reports transport connectivity only. This hook adds
 * runtime audio phase and diagnostics from `getAudioStatus` + `audioStatusChanged`.
 */

import { useMemo, useState } from 'react';
import { logger } from '../logger/Logger';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcEvents } from '../ipc/constants';
import {
  isAudioRuntimeStatus,
  type AudioDiagnostic,
  type AudioRuntimePhase,
  type AudioRuntimeStatus,
} from '../types/ipc';
import { _usePollingSubscription } from './_usePollingSubscription';

export interface UseAudioStatusResult {
  /** Runtime audio phase. `null` means status unavailable/disconnected. */
  phase: AudioRuntimePhase | null;
  /** Optional structured diagnostic for degraded/failed states. */
  diagnostic: AudioDiagnostic | undefined;
  /** True when audio is running (full-duplex or input-only). */
  isReady: boolean;
  /** True for degraded mode or hard failure. */
  isDegraded: boolean;
  /** Raw status payload from Rust side. */
  status: AudioRuntimeStatus | null;
}

export function useAudioStatus(): UseAudioStatusResult {
  const [status, setStatus] = useState<AudioRuntimeStatus | null>(null);

  _usePollingSubscription(() => {
    const bridge = IpcBridge.getInstance();
    let mounted = true;

    const fetchStatus = async (): Promise<void> => {
      if (!bridge.isConnected()) {
        return;
      }

      try {
        const next = await bridge.getAudioStatus();
        if (mounted) {
          setStatus(next);
        }
      } catch (error) {
        if (!mounted || !bridge.isConnected()) {
          return;
        }

        const message = error instanceof Error ? error.message : String(error);
        logger.error('getAudioStatus failed while transport is connected', { error });

        setStatus({
          phase: 'failed',
          diagnostic: {
            code: 'unknown',
            message: `getAudioStatus failed: ${message}`,
            hint: 'Ensure the dev server and plugin runtime expose getAudioStatus.',
          },
          updated_at_ms: Date.now(),
        });
      }
    };

    const unsubscribeConnection = bridge.onConnectionChange((connected) => {
      if (!connected) {
        if (mounted) {
          setStatus(null);
        }
        return;
      }

      void fetchStatus();
    });

    const unsubscribeStatus = bridge.on<unknown>(IpcEvents.AUDIO_STATUS_CHANGED, (payload) => {
      if (!mounted) {
        return;
      }

      if (isAudioRuntimeStatus(payload)) {
        setStatus(payload);
      } else {
        logger.warn('Received malformed audioStatusChanged payload', { payload });
      }
    });

    // Initial fetch for already-connected transports.
    void fetchStatus();

    return () => {
      mounted = false;
      unsubscribeConnection();
      unsubscribeStatus();
    };
  }, []);

  return useMemo(() => {
    const phase = status?.phase ?? null;
    const isReady = phase === 'runningFullDuplex' || phase === 'runningInputOnly';
    const isDegraded = phase === 'degraded' || phase === 'failed';

    return {
      phase,
      diagnostic: status?.diagnostic,
      isReady,
      isDegraded,
      status,
    };
  }, [status]);
}
