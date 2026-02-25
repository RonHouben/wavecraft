import { useEffect, useMemo, useRef, useState } from 'react';
import type { MeterFrame } from '../types/metering';
import { useMeterFrame } from './useMeterFrame';

const DEFAULT_INTERVAL_MS = 50;
const DEFAULT_THRESHOLD = 0.002;
const DEFAULT_HOLD_MS = 220;

export interface MeterSignalActivitySmoothing {
  readonly enabled: boolean;
  readonly holdMs?: number;
}

export interface UseMeterSignalActivityOptions {
  readonly intervalMs?: number;
  readonly threshold?: number;
  readonly smoothing?: MeterSignalActivitySmoothing;
}

export function useMeterSignalActivity(options: Readonly<UseMeterSignalActivityOptions> = {}): {
  isSignalActive: boolean;
} {
  const intervalMs = options.intervalMs ?? DEFAULT_INTERVAL_MS;
  const threshold = options.threshold ?? DEFAULT_THRESHOLD;
  const smoothingEnabled = options.smoothing?.enabled ?? false;
  const holdMs = options.smoothing?.holdMs ?? DEFAULT_HOLD_MS;

  const frame = useMeterFrame(intervalMs);
  const signalLevel = useMemo(() => getMeterSignalLevel(frame), [frame]);
  const aboveThreshold = signalLevel >= threshold;
  const [isSignalActive, setIsSignalActive] = useState(aboveThreshold);
  const signalIdleTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!smoothingEnabled) {
      if (signalIdleTimeoutRef.current !== null) {
        clearTimeout(signalIdleTimeoutRef.current);
        signalIdleTimeoutRef.current = null;
      }

      setIsSignalActive(aboveThreshold);
      return;
    }

    if (aboveThreshold) {
      setIsSignalActive(true);

      if (signalIdleTimeoutRef.current !== null) {
        clearTimeout(signalIdleTimeoutRef.current);
        signalIdleTimeoutRef.current = null;
      }

      return;
    }

    if (!isSignalActive || signalIdleTimeoutRef.current !== null) {
      return;
    }

    signalIdleTimeoutRef.current = globalThis.setTimeout(() => {
      setIsSignalActive(false);
      signalIdleTimeoutRef.current = null;
    }, holdMs);
  }, [aboveThreshold, holdMs, isSignalActive, smoothingEnabled]);

  useEffect(() => {
    return (): void => {
      if (signalIdleTimeoutRef.current !== null) {
        clearTimeout(signalIdleTimeoutRef.current);
      }
    };
  }, []);

  return { isSignalActive };
}

export function getMeterSignalLevel(frame: MeterFrame | null | undefined): number {
  return Math.max(frame?.peak_l ?? 0, frame?.peak_r ?? 0, frame?.rms_l ?? 0, frame?.rms_r ?? 0);
}
