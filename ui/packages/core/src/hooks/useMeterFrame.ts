/**
 * useMeterFrame - Hook for polling audio meter data
 */

import { useState } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcMethods } from '../ipc/constants';
import type { MeterFrame, GetMeterFrameResult } from '../types/metering';
import { _usePollingSubscription } from './_usePollingSubscription';

/**
 * Hook to poll meter frames at a specified interval
 *
 * @param intervalMs - Polling interval in milliseconds (default: 50ms = 20fps)
 * @returns Current meter frame or null if not available
 */
export function useMeterFrame(intervalMs = 50): MeterFrame | null {
  const [frame, setFrame] = useState<MeterFrame | null>(null);

  _usePollingSubscription(() => {
    let isMounted = true;
    const bridge = IpcBridge.getInstance();

    async function fetchFrame(): Promise<void> {
      if (!bridge.isConnected()) return;

      try {
        const result = await bridge.invoke<GetMeterFrameResult>(IpcMethods.GET_METER_FRAME);
        if (isMounted && result.frame) {
          setFrame(result.frame);
        }
      } catch {
        // Silently ignore meter fetch errors
      }
    }

    // Initial fetch
    fetchFrame();

    // Periodic polling
    const intervalId = setInterval(fetchFrame, intervalMs);

    return (): void => {
      isMounted = false;
      clearInterval(intervalId);
    };
  }, [intervalMs]);

  return frame;
}
