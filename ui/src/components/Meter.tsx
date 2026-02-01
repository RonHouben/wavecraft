/**
 * Meter - Audio level meter visualization component
 *
 * Displays peak and RMS levels for stereo audio with dB scaling
 */

import React, { useEffect, useState, useRef } from 'react';
import { getMeterFrame, linearToDb, useConnectionStatus, type MeterFrame } from '../lib/vstkit-ipc';

const METER_UPDATE_HZ = 30;
const METER_FLOOR_DB = -60;
const METER_RANGE_DB = 60; // 0 to -60 dB
const CLIP_THRESHOLD = 1; // Linear amplitude threshold
const CLIP_HOLD_MS = 2000; // Hold clip indicator for 2 seconds

export function Meter(): React.JSX.Element {
  const { connected } = useConnectionStatus();
  const [frame, setFrame] = useState<MeterFrame | null>(null);
  const [clippedL, setClippedL] = useState(false);
  const [clippedR, setClippedR] = useState(false);

  const clipLTimeoutRef = useRef<number | null>(null);
  const clipRTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    // Only poll when connected
    if (!connected) {
      return;
    }

    // Poll meter frames at 30 Hz
    const interval = setInterval(async () => {
      const newFrame = await getMeterFrame();
      setFrame(newFrame);

      // Detect clipping (peak > 1.0 linear = 0 dB)
      if (newFrame) {
        if (newFrame.peak_l > CLIP_THRESHOLD) {
          setClippedL(true);
          // Clear existing timeout and set new one
          if (clipLTimeoutRef.current !== null) {
            clearTimeout(clipLTimeoutRef.current);
          }
          clipLTimeoutRef.current = globalThis.setTimeout(() => {
            setClippedL(false);
            clipLTimeoutRef.current = null;
          }, CLIP_HOLD_MS);
        }

        if (newFrame.peak_r > CLIP_THRESHOLD) {
          setClippedR(true);
          // Clear existing timeout and set new one
          if (clipRTimeoutRef.current !== null) {
            clearTimeout(clipRTimeoutRef.current);
          }
          clipRTimeoutRef.current = globalThis.setTimeout(() => {
            setClippedR(false);
            clipRTimeoutRef.current = null;
          }, CLIP_HOLD_MS);
        }
      }
    }, 1000 / METER_UPDATE_HZ);

    return (): void => {
      clearInterval(interval);
      if (clipLTimeoutRef.current !== null) {
        clearTimeout(clipLTimeoutRef.current);
      }
      if (clipRTimeoutRef.current !== null) {
        clearTimeout(clipRTimeoutRef.current);
      }
    };
  }, [connected]);

  // Convert linear to dB for display
  const peakLDb = frame ? linearToDb(frame.peak_l, METER_FLOOR_DB) : METER_FLOOR_DB;
  const peakRDb = frame ? linearToDb(frame.peak_r, METER_FLOOR_DB) : METER_FLOOR_DB;
  const rmsLDb = frame ? linearToDb(frame.rms_l, METER_FLOOR_DB) : METER_FLOOR_DB;
  const rmsRDb = frame ? linearToDb(frame.rms_r, METER_FLOOR_DB) : METER_FLOOR_DB;

  // Normalize to 0-100% for CSS
  const peakLPercent = ((peakLDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;
  const peakRPercent = ((peakRDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;
  const rmsLPercent = ((rmsLDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;
  const rmsRPercent = ((rmsRDb - METER_FLOOR_DB) / METER_RANGE_DB) * 100;

  const handleResetClip = (): void => {
    setClippedL(false);
    setClippedR(false);
    if (clipLTimeoutRef.current !== null) {
      clearTimeout(clipLTimeoutRef.current);
      clipLTimeoutRef.current = null;
    }
    if (clipRTimeoutRef.current !== null) {
      clearTimeout(clipRTimeoutRef.current);
      clipRTimeoutRef.current = null;
    }
  };

  // Show connecting state when not connected
  if (!connected) {
    return (
      <div className="flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4 font-sans">
        <div className="flex items-center justify-between gap-2">
          <div className="text-xs font-semibold uppercase tracking-wide text-gray-500">Levels</div>
        </div>
        <div className="flex items-center justify-center py-8 text-sm text-gray-400">
          ⏳ Connecting...
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4 font-sans">
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs font-semibold uppercase tracking-wide text-gray-500">Levels</div>
        {(clippedL || clippedR) && (
          <button
            className="animate-clip-pulse cursor-pointer select-none rounded border-none bg-meter-clip px-2 py-0.5 text-[10px] font-bold text-white hover:bg-meter-clip-dark active:scale-95"
            onClick={handleResetClip}
            title="Click to reset"
            type="button"
          >
            CLIP
          </button>
        )}
      </div>

      <div className="flex items-center gap-2 rounded bg-plugin-dark p-2">
        <div className="w-4 text-center text-[11px] font-semibold text-gray-300">L</div>
        <div className="relative h-6 flex-1">
          <div
            className={`relative h-full w-full overflow-hidden rounded bg-[#333] transition-shadow duration-100 ${
              clippedL ? 'shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]' : ''
            }`}
          >
            <div
              className="absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light transition-[width] duration-100"
              style={{ width: `${Math.max(0, Math.min(100, rmsLPercent))}%` }}
            />
            <div
              className="duration-50 absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 transition-[width]"
              style={{ width: `${Math.max(0, Math.min(100, peakLPercent))}%` }}
            />
          </div>
        </div>
        <div
          className={`w-[60px] text-right font-mono text-[11px] text-gray-300 transition-colors duration-100 ${
            clippedL ? 'font-semibold text-meter-clip' : ''
          }`}
        >
          {peakLDb.toFixed(1)} dB
        </div>
      </div>

      <div className="flex items-center gap-2 rounded bg-plugin-dark p-2">
        <div className="w-4 text-center text-[11px] font-semibold text-gray-300">R</div>
        <div className="relative h-6 flex-1">
          <div
            className={`relative h-full w-full overflow-hidden rounded bg-[#333] transition-shadow duration-100 ${
              clippedR ? 'shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]' : ''
            }`}
          >
            <div
              className="absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light transition-[width] duration-100"
              style={{ width: `${Math.max(0, Math.min(100, rmsRPercent))}%` }}
            />
            <div
              className="duration-50 absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 transition-[width]"
              style={{ width: `${Math.max(0, Math.min(100, peakRPercent))}%` }}
            />
          </div>
        </div>
        <div
          className={`w-[60px] text-right font-mono text-[11px] text-gray-300 transition-colors duration-100 ${
            clippedR ? 'font-semibold text-meter-clip' : ''
          }`}
        >
          {peakRDb.toFixed(1)} dB
        </div>
      </div>
    </div>
  );
}
