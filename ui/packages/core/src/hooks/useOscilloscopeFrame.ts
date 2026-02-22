/**
 * useOscilloscopeFrame - Hook for requestAnimationFrame-driven waveform polling
 */

import { useState } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { METHOD_GET_OSCILLOSCOPE_FRAME } from '../types/ipc';
import type { GetOscilloscopeFrameResult, OscilloscopeFrame } from '../types/oscilloscope';
import { _usePollingSubscription } from './_usePollingSubscription';

/**
 * Hook to poll oscilloscope frames on the browser animation frame cadence.
 *
 * Uses single in-flight request semantics to avoid transport backpressure.
 */
export function useOscilloscopeFrame(): OscilloscopeFrame | null {
  const [frame, setFrame] = useState<OscilloscopeFrame | null>(null);

  _usePollingSubscription(() => {
    let isMounted = true;
    let inFlight = false;
    let rafId: number | null = null;
    const bridge = IpcBridge.getInstance();

    const tick = async (): Promise<void> => {
      if (!isMounted) {
        return;
      }

      if (bridge.isConnected() && !inFlight) {
        inFlight = true;
        try {
          const result = await bridge.invoke<GetOscilloscopeFrameResult>(
            METHOD_GET_OSCILLOSCOPE_FRAME
          );
          if (isMounted && result.frame) {
            setFrame(result.frame);
          }
        } catch {
          // Silently ignore oscilloscope fetch errors
        } finally {
          inFlight = false;
        }
      }

      rafId = requestAnimationFrame(() => {
        void tick();
      });
    };

    void tick();

    return (): void => {
      isMounted = false;
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
    };
  }, []);

  return frame;
}
