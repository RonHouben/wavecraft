/**
 * Meter - Audio level meter visualization component
 *
 * Displays peak and RMS levels for stereo audio with dB scaling
 */

import { useConnectionStatus, useMeterFrame } from '@wavecraft/core';
import React, { useEffect, useRef, useState } from 'react';
import { Card } from './Card';
import type { ControlVisualState, PluginVisualState } from './types';
import {
  elevatedCardClass,
  focusRingClass,
  insetSurfaceClass,
  mergeClassNames,
  statusChipClass,
} from './utils/classNames';
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
  readonly className?: string;
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
      className={mergeClassNames(insetSurfaceClass, 'flex items-center gap-3 px-3 py-2')}
    >
      <div className="w-5 text-center text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
        {side}
      </div>
      <div className="relative h-7 flex-1">
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
        className={`w-16 text-right font-mono text-type-sm text-plugin-text-primary motion-safe:transition-colors motion-safe:duration-100 ${
          clipped ? 'font-semibold text-meter-clip' : ''
        }`}
      >
        {peakDb.toFixed(1)} dB
      </div>
    </div>
  );
}

export function Meter({
  className,
  pluginState,
  state = 'default',
}: Readonly<MeterProps>): React.JSX.Element {
  const frame = useMeterFrame(50);
  const { connected } = useConnectionStatus();

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
  const meterClassName = mergeClassNames(
    'font-sans transition-[opacity,filter] duration-150',
    elevatedCardClass,
    getControlStateClass({ pluginState, state }),
    isError ? 'border-meter-clip' : '',
    isBypassed ? 'opacity-70 brightness-90 saturate-50' : 'opacity-100 saturate-100',
    className
  );

  const handleResetClip = (): void => {
    resetHandlersRef.current.L();
    resetHandlersRef.current.R();
  };

  // Show connecting/loading state
  if (isLoading) {
    return (
      <Card
        data-testid="meter"
        data-state={state}
        data-plugin-state={pluginState}
        className={meterClassName}
      >
        <Card.Header>
          <Card.Title>Levels</Card.Title>
          {badgeLabel ? (
            <span
              className={mergeClassNames(
                statusChipClass,
                'font-mono leading-none',
                getStateBadgeClass(pluginState)
              )}
              aria-hidden="true"
            >
              {badgeLabel}
            </span>
          ) : null}
        </Card.Header>
        <Card.Content>
          <div
            className={mergeClassNames(
              insetSurfaceClass,
              'flex items-center justify-center py-8 text-type-sm text-plugin-text-secondary'
            )}
          >
            ⏳ {isError ? 'Meter unavailable' : 'Connecting...'}
          </div>
        </Card.Content>
      </Card>
    );
  }

  return (
    <Card
      data-testid="meter"
      data-state={state}
      data-plugin-state={pluginState}
      className={meterClassName}
    >
      <Card.Header>
        <Card.Title>Levels</Card.Title>
        <div className="flex items-center gap-2">
          {badgeLabel ? (
            <span
              className={mergeClassNames(
                statusChipClass,
                'font-mono leading-none',
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
              className={mergeClassNames(
                'animate-clip-pulse cursor-pointer select-none rounded-md border border-meter-clip-dark bg-meter-clip px-2 py-1 text-type-2xs font-semibold uppercase tracking-wide text-white hover:bg-meter-clip-dark active:scale-95',
                focusRingClass
              )}
              onClick={handleResetClip}
              title="Click to reset"
              type="button"
            >
              Clip
            </button>
          )}
        </div>
      </Card.Header>

      <Card.Content className="flex flex-col gap-3">
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
      </Card.Content>
    </Card>
  );
}
