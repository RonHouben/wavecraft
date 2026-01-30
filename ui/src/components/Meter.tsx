/**
 * Meter - Audio level meter visualization component
 * 
 * Displays peak and RMS levels for stereo audio with dB scaling
 */

import { useEffect, useState, useRef } from 'react';
import { getMeterFrame, linearToDb, type MeterFrame } from '../lib/vstkit-ipc';
import './Meter.css';

const METER_UPDATE_HZ = 30;
const METER_FLOOR_DB = -60;
const METER_RANGE_DB = 60; // 0 to -60 dB
const CLIP_THRESHOLD = 1; // Linear amplitude threshold
const CLIP_HOLD_MS = 2000; // Hold clip indicator for 2 seconds

export function Meter() {
  const [frame, setFrame] = useState<MeterFrame | null>(null);
  const [clippedL, setClippedL] = useState(false);
  const [clippedR, setClippedR] = useState(false);
  
  const clipLTimeoutRef = useRef<number | null>(null);
  const clipRTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
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

    return () => {
      clearInterval(interval);
      if (clipLTimeoutRef.current !== null) {
        clearTimeout(clipLTimeoutRef.current);
      }
      if (clipRTimeoutRef.current !== null) {
        clearTimeout(clipRTimeoutRef.current);
      }
    };
  }, []);

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
  
  const handleResetClip = () => {
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

  return (
    <div className="meter">
      <div className="meter-header">
        <div className="meter-label">Levels</div>
        {(clippedL || clippedR) && (
          <button 
            className="meter-clip-indicator" 
            onClick={handleResetClip}
            title="Click to reset"
            type="button"
          >
            CLIP
          </button>
        )}
      </div>
      
      <div className="meter-channel">
        <div className="meter-channel-label">L</div>
        <div className="meter-bar-container">
          <div className={`meter-bar-bg ${clippedL ? 'clipped' : ''}`}>
            <div
              className="meter-bar-rms"
              style={{ width: `${Math.max(0, Math.min(100, rmsLPercent))}%` }}
            />
            <div
              className="meter-bar-peak"
              style={{ width: `${Math.max(0, Math.min(100, peakLPercent))}%` }}
            />
          </div>
        </div>
        <div className={`meter-value ${clippedL ? 'clipped' : ''}`}>
          {peakLDb.toFixed(1)} dB
        </div>
      </div>

      <div className="meter-channel">
        <div className="meter-channel-label">R</div>
        <div className="meter-bar-container">
          <div className={`meter-bar-bg ${clippedR ? 'clipped' : ''}`}>
            <div
              className="meter-bar-rms"
              style={{ width: `${Math.max(0, Math.min(100, rmsRPercent))}%` }}
            />
            <div
              className="meter-bar-peak"
              style={{ width: `${Math.max(0, Math.min(100, peakRPercent))}%` }}
            />
          </div>
        </div>
        <div className={`meter-value ${clippedR ? 'clipped' : ''}`}>
          {peakRDb.toFixed(1)} dB
        </div>
      </div>
    </div>
  );
}
