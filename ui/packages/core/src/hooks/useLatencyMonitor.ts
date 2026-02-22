/**
 * useLatencyMonitor - Hook for monitoring IPC latency
 */

import { useState } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import { logger } from '../logger';
import { _usePollingSubscription } from './_usePollingSubscription';

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

  _usePollingSubscription(() => {
    let isMounted = true;
    const client = ParameterClient.getInstance();

    async function measure(): Promise<void> {
      // Only measure when connected
      if (!bridge.isConnected()) {
        return;
      }

      try {
        const ms = await client.ping();
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
