/**
 * Meter - Audio level meter visualization component
 *
 * Displays peak and RMS levels for stereo audio with dB scaling
 */

import React, { useEffect, useState, useRef } from 'react';
import type { ControlVisualState, MeterFrame, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames, surfaceCardClass } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

const METER_FLOOR_DB = -60;
const METER_RANGE_DB = 60; // 0 to -60 dB
const CLIP_THRESHOLD = 1; // Linear amplitude threshold
const CLIP_HOLD_MS = 2000; // Hold clip indicator for 2 seconds

function linearToDb(linear: number, floorDb = METER_FLOOR_DB): number {
  if (linear <= 0) {
    return floorDb;
  }
  return Math.max(floorDb, 20 * Math.log10(linear));
}

export interface MeterProps {
  readonly connected: boolean;
  readonly frame: MeterFrame | null;
  readonly state?: ControlVisualState;
  readonly pluginState?: PluginVisualState;
}

interface MeterChannelProps {
  readonly side: 'L' | 'R';
  readonly peakLinear: number;
  readonly rmsLinear: number;
  readonly onClippedChange: (side: 'L' | 'R', clipped: boolean) => void;
  readonly onRegisterReset: (side: 'L' | 'R', reset: () => void) => void;
}

function MeterChannel({
  side,
  peakLinear,
  rmsLinear,
  onClippedChange,
  onRegisterReset,
}: Readonly<MeterChannelProps>): React.JSX.Element {
  const [clipped, setClipped] = useState(false);
  const clipTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (peakLinear <= CLIP_THRESHOLD) {
      return;
    }

    setClipped(true);
    if (clipTimeoutRef.current !== null) {
      clearTimeout(clipTimeoutRef.current);
    }

    clipTimeoutRef.current = globalThis.setTimeout(() => {
      setClipped(false);
      clipTimeoutRef.current = null;
    }, CLIP_HOLD_MS);
  }, [peakLinear]);

  useEffect(() => {
    onClippedChange(side, clipped);
  }, [clipped, onClippedChange, side]);

  const resetClip = React.useCallback((): void => {
    setClipped(false);
    if (clipTimeoutRef.current !== null) {
      clearTimeout(clipTimeoutRef.current);
      clipTimeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    onRegisterReset(side, resetClip);
  }, [onRegisterReset, resetClip, side]);

  useEffect(() => {
    return (): void => {
      if (clipTimeoutRef.current !== null) {
        clearTimeout(clipTimeoutRef.current);
      }
      onClippedChange(side, false);
      onRegisterReset(side, () => {});
    };
  }, [onClippedChange, onRegisterReset, side]);

  const peakDb = linearToDb(peakLinear, METER_FLOOR_DB);
  const rmsDb = linearToDb(rmsLinear, METER_FLOOR_DB);
  const peakPercent = ((peakDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;
  const rmsPercent = ((rmsDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;

  return (
    <div
      data-testid={`meter-${side}`}
      className="flex items-center gap-2 rounded bg-plugin-dark p-2"
    >
      <div className="w-4 text-center text-[11px] font-semibold text-gray-300">{side}</div>
      <div className="relative h-6 flex-1">
        <div
          className={`relative h-full w-full overflow-hidden rounded bg-plugin-surface motion-safe:transition-shadow motion-safe:duration-100 ${
            clipped ? 'shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]' : ''
          }`}
        >
          <div
            data-testid={`meter-${side}-rms`}
            className="absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light motion-safe:transition-[width] motion-safe:duration-100"
            style={{ width: `${Math.max(0, Math.min(100, rmsPercent))}%` }}
          />
          <div
            data-testid={`meter-${side}-peak`}
            className="absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 motion-safe:transition-[width] motion-safe:duration-75"
            style={{ width: `${Math.max(0, Math.min(100, peakPercent))}%` }}
          />
        </div>
      </div>
      <div
        data-testid={`meter-${side}-db`}
        className={`w-[60px] text-right font-mono text-[11px] text-gray-300 motion-safe:transition-colors motion-safe:duration-100 ${
          clipped ? 'font-semibold text-meter-clip' : ''
        }`}
      >
        {peakDb.toFixed(1)} dB
      </div>
    </div>
  );
}

export function Meter({
  connected,
  frame,
  pluginState,
  state = 'default',
}: Readonly<MeterProps>): React.JSX.Element {
  const [channelClippedState, setChannelClippedState] = useState<Record<'L' | 'R', boolean>>({
    L: false,
    R: false,
  });
  const resetHandlersRef = useRef<Record<'L' | 'R', () => void>>({
    L: () => {},
    R: () => {},
  });

  const handleClippedChange = React.useCallback((side: 'L' | 'R', clipped: boolean): void => {
    setChannelClippedState((prev) => {
      if (prev[side] === clipped) {
        return prev;
      }

      return {
        ...prev,
        [side]: clipped,
      };
    });
  }, []);

  const handleRegisterReset = React.useCallback((side: 'L' | 'R', reset: () => void): void => {
    resetHandlersRef.current[side] = reset;
  }, []);

  const clippedL = channelClippedState.L;
  const clippedR = channelClippedState.R;
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isLoading = state === 'loading' || !connected;
  const isError = state === 'error';
  const isBypassed = pluginState === 'bypassed';

  const handleResetClip = (): void => {
    resetHandlersRef.current.L();
    resetHandlersRef.current.R();
  };

  // Show connecting/loading state
  if (isLoading) {
    return (
      <div
        data-testid="meter"
        data-state={state}
        data-plugin-state={pluginState}
        className={mergeClassNames(
          'flex flex-col gap-2 font-sans',
          surfaceCardClass,
          getControlStateClass({ state, pluginState }),
          isError ? 'border-meter-clip' : ''
        )}
      >
        <div className="flex items-center justify-between gap-2">
          <div className="text-xs font-semibold uppercase tracking-wide text-gray-500">Levels</div>
          {badgeLabel ? (
            <span
              className={mergeClassNames(
                'rounded-sm border px-1 py-0.5 font-mono text-[10px] leading-none',
                getStateBadgeClass(pluginState)
              )}
              aria-hidden="true"
            >
              {badgeLabel}
            </span>
          ) : null}
        </div>
        <div className="flex items-center justify-center py-8 text-sm text-gray-400">
          ⏳ {isError ? 'Meter unavailable' : 'Connecting...'}
        </div>
      </div>
    );
  }

  return (
    <div
      data-testid="meter"
      data-state={state}
      data-plugin-state={pluginState}
      className={mergeClassNames(
        'flex flex-col gap-2 font-sans',
        surfaceCardClass,
        getControlStateClass({ pluginState, state }),
        isError ? 'border-meter-clip' : '',
        isBypassed ? 'opacity-70' : ''
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs font-semibold uppercase tracking-wide text-gray-500">Levels</div>
        {badgeLabel ? (
          <span
            className={mergeClassNames(
              'rounded-sm border px-1 py-0.5 font-mono text-[10px] leading-none',
              getStateBadgeClass(pluginState)
            )}
            aria-hidden="true"
          >
            {badgeLabel}
          </span>
        ) : null}
        {(clippedL || clippedR) && (
          <button
            data-testid="meter-clip-button"
            className={`animate-clip-pulse cursor-pointer select-none rounded border-none bg-meter-clip px-2 py-0.5 text-[10px] font-bold text-white hover:bg-meter-clip-dark active:scale-95 ${focusRingClass}`}
            onClick={handleResetClip}
            title="Click to reset"
            type="button"
          >
            CLIP
          </button>
        )}
      </div>

      <MeterChannel
        side="L"
        peakLinear={frame?.peak_l ?? 0}
        rmsLinear={frame?.rms_l ?? 0}
        onClippedChange={handleClippedChange}
        onRegisterReset={handleRegisterReset}
      />
      <MeterChannel
        side="R"
        peakLinear={frame?.peak_r ?? 0}
        rmsLinear={frame?.rms_r ?? 0}
        onClippedChange={handleClippedChange}
        onRegisterReset={handleRegisterReset}
      />
    </div>
  );
}
