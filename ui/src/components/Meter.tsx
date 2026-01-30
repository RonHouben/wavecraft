/**
 * Meter - Audio level meter visualization component
 * 
 * Displays peak and RMS levels for stereo audio with dB scaling
 */

import { useEffect, useState } from 'react';
import { getMeterFrame, linearToDb, type MeterFrame } from '../lib/vstkit-ipc';
import './Meter.css';

const METER_UPDATE_HZ = 30;
const METER_FLOOR_DB = -60;
const METER_RANGE_DB = 60; // 0 to -60 dB

export function Meter() {
  const [frame, setFrame] = useState<MeterFrame | null>(null);

  useEffect(() => {
    // Poll meter frames at 30 Hz
    const interval = setInterval(async () => {
      const newFrame = await getMeterFrame();
      setFrame(newFrame);
    }, 1000 / METER_UPDATE_HZ);

    return () => clearInterval(interval);
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

  return (
    <div className="meter">
      <div className="meter-label">Levels</div>
      
      <div className="meter-channel">
        <div className="meter-channel-label">L</div>
        <div className="meter-bar-container">
          <div className="meter-bar-bg">
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
        <div className="meter-value">{peakLDb.toFixed(1)} dB</div>
      </div>

      <div className="meter-channel">
        <div className="meter-channel-label">R</div>
        <div className="meter-bar-container">
          <div className="meter-bar-bg">
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
        <div className="meter-value">{peakRDb.toFixed(1)} dB</div>
      </div>
    </div>
  );
}
