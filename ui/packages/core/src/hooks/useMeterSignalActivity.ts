import { useEffect, useMemo, useRef, useState } from 'react';
import type { MeterFrame } from '../types/metering';
import { linearToDb } from '../utils/audio-math';
import { useMeterFrame } from './useMeterFrame';
import { usePassthroughMeterFrame } from './usePassthroughMeterFrame';

const DEFAULT_INTERVAL_MS = 50;
const DEFAULT_THRESHOLD = 0.002;
const DEFAULT_HOLD_MS = 220;
const DEFAULT_INTENSITY_FLOOR_DB = -48;
const DEFAULT_INTENSITY_CEILING_DB = 0;
const DEFAULT_CLIP_WARNING_THRESHOLD_DB = -1;
const DEFAULT_CLIP_WARNING_CEILING_DB = 0;

export interface MeterSignalActivitySmoothing {
  readonly enabled: boolean;
  readonly holdMs?: number;
}

export interface UseMeterSignalActivityOptions {
  readonly intervalMs?: number;
  readonly threshold?: number;
  readonly smoothing?: MeterSignalActivitySmoothing;
  readonly intensityRange?: MeterSignalIntensityRange;
}

export interface MeterSignalIntensityRange {
  readonly floorDb?: number;
  readonly ceilingDb?: number;
}

export interface MeterClipWarningRange {
  readonly thresholdDb?: number;
  readonly ceilingDb?: number;
}

export interface MeterSignalActivityState {
  isSignalActive: boolean;
  signalLevel: number;
  signalIntensity: number;
}

export function useMeterSignalActivity(
  options: Readonly<UseMeterSignalActivityOptions> = {}
): MeterSignalActivityState {
  const intervalMs = options.intervalMs ?? DEFAULT_INTERVAL_MS;
  const frame = useMeterFrame(intervalMs);
  return useSignalActivityForFrame(frame, options);
}

export function usePassthroughMeterSignalActivity(
  options: Readonly<UseMeterSignalActivityOptions> = {}
): MeterSignalActivityState {
  const intervalMs = options.intervalMs ?? DEFAULT_INTERVAL_MS;
  const frame = usePassthroughMeterFrame(intervalMs);
  return useSignalActivityForFrame(frame, options);
}

function useSignalActivityForFrame(
  frame: MeterFrame | null,
  options: Readonly<UseMeterSignalActivityOptions>
): MeterSignalActivityState {
  const threshold = options.threshold ?? DEFAULT_THRESHOLD;
  const smoothingEnabled = options.smoothing?.enabled ?? false;
  const holdMs = options.smoothing?.holdMs ?? DEFAULT_HOLD_MS;
  const intensityFloorDb = options.intensityRange?.floorDb ?? DEFAULT_INTENSITY_FLOOR_DB;
  const intensityCeilingDb = options.intensityRange?.ceilingDb ?? DEFAULT_INTENSITY_CEILING_DB;

  const signalLevel = useMemo(() => getMeterSignalLevel(frame), [frame]);
  const signalIntensity = useMemo(
    () =>
      getMeterSignalIntensity(signalLevel, {
        floorDb: intensityFloorDb,
        ceilingDb: intensityCeilingDb,
      }),
    [intensityCeilingDb, intensityFloorDb, signalLevel]
  );
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

  return { isSignalActive, signalLevel, signalIntensity };
}

export function getMeterSignalLevel(frame: MeterFrame | null | undefined): number {
  return Math.max(frame?.peak_l ?? 0, frame?.peak_r ?? 0, frame?.rms_l ?? 0, frame?.rms_r ?? 0);
}

export function getMeterSignalIntensity(
  signalLevel: number,
  range: Readonly<MeterSignalIntensityRange> = {}
): number {
  if (!Number.isFinite(signalLevel) || signalLevel <= 0) {
    return 0;
  }

  const floorDb =
    typeof range.floorDb === 'number' && Number.isFinite(range.floorDb)
      ? range.floorDb
      : DEFAULT_INTENSITY_FLOOR_DB;
  const ceilingDb =
    typeof range.ceilingDb === 'number' && Number.isFinite(range.ceilingDb)
      ? range.ceilingDb
      : DEFAULT_INTENSITY_CEILING_DB;

  const minDb = Math.min(floorDb, ceilingDb);
  const maxDb = Math.max(floorDb, ceilingDb);

  if (maxDb === minDb) {
    return signalLevel > 0 ? 1 : 0;
  }

  const signalDb = linearToDb(signalLevel, minDb);
  const boundedDb = clamp(signalDb, minDb, maxDb);

  return clamp((boundedDb - minDb) / (maxDb - minDb), 0, 1);
}

export function getMeterClipWarningIntensity(
  signalLevel: number,
  range: Readonly<MeterClipWarningRange> = {}
): number {
  if (!Number.isFinite(signalLevel) || signalLevel <= 0) {
    return 0;
  }

  const thresholdDb =
    typeof range.thresholdDb === 'number' && Number.isFinite(range.thresholdDb)
      ? range.thresholdDb
      : DEFAULT_CLIP_WARNING_THRESHOLD_DB;
  const ceilingDb =
    typeof range.ceilingDb === 'number' && Number.isFinite(range.ceilingDb)
      ? range.ceilingDb
      : DEFAULT_CLIP_WARNING_CEILING_DB;

  const minDb = Math.min(thresholdDb, ceilingDb);
  const maxDb = Math.max(thresholdDb, ceilingDb);

  if (maxDb === minDb) {
    return signalLevel >= 1 ? 1 : 0;
  }

  const signalDb = linearToDb(signalLevel, minDb);
  const boundedDb = clamp(signalDb, minDb, maxDb);

  return clamp((boundedDb - minDb) / (maxDb - minDb), 0, 1);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
