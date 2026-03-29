/**
 * usePassthroughMeterFrame - Hook for polling Passthrough-local meter data
 */

import { useState } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { IpcMethods } from '../ipc/constants';
import type { GetMeterFrameResult, MeterFrame } from '../types/metering';
import { _usePollingSubscription } from './_usePollingSubscription';

/**
 * Hook to poll Passthrough-local meter frames at a specified interval.
 *
 * @param intervalMs - Polling interval in milliseconds (default: 50ms = 20fps)
 * @returns Current Passthrough-local meter frame or null if not available
 */
export function usePassthroughMeterFrame(intervalMs = 50): MeterFrame | null {
  const [frame, setFrame] = useState<MeterFrame | null>(null);

  _usePollingSubscription(() => {
    let isMounted = true;
    const bridge = IpcBridge.getInstance();

    async function fetchFrame(): Promise<void> {
      if (!bridge.isConnected()) return;

      try {
        const result = await bridge.invoke<GetMeterFrameResult>(
          IpcMethods.GET_PASSTHROUGH_METER_FRAME
        );
        if (isMounted && result.frame) {
          setFrame(result.frame);
        }
      } catch {
        // Silently ignore meter fetch errors
      }
    }

    fetchFrame();

    const intervalId = setInterval(fetchFrame, intervalMs);

    return (): void => {
      isMounted = false;
      clearInterval(intervalId);
    };
  }, [intervalMs]);

  return frame;
}
