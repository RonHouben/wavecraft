/**
 * LatencyMonitor - Displays IPC roundtrip latency metrics
 */

import React from 'react';
import { useLatencyMonitor } from '@vstkit/ipc';
import './LatencyMonitor.css';

export function LatencyMonitor(): React.JSX.Element {
  const { latency, avg, max, count } = useLatencyMonitor(1000);

  return (
    <div className="latency-monitor">
      <h3>IPC Latency</h3>
      <div className="metrics">
        <div className="metric">
          <span className="label">Current:</span>
          <span className="value">{latency !== null ? `${latency.toFixed(2)} ms` : '—'}</span>
        </div>
        <div className="metric">
          <span className="label">Average:</span>
          <span className="value">{avg > 0 ? `${avg.toFixed(2)} ms` : '—'}</span>
        </div>
        <div className="metric">
          <span className="label">Max:</span>
          <span className="value">{max > 0 ? `${max.toFixed(2)} ms` : '—'}</span>
        </div>
        <div className="metric">
          <span className="label">Samples:</span>
          <span className="value">{count}</span>
        </div>
      </div>
      <div className="status">
        {avg > 0 && (
          <span className={avg < 5 ? 'good' : avg < 10 ? 'warning' : 'poor'}>
            {avg < 5 ? '✓ Excellent' : avg < 10 ? '⚠ Fair' : '✗ Poor'}
          </span>
        )}
      </div>
    </div>
  );
}
