/**
 * LatencyMonitor - Displays IPC roundtrip latency metrics
 */

import React from 'react';

export interface LatencyMonitorProps {
  readonly latency: number | null;
  readonly avg: number;
  readonly max: number;
  readonly count: number;
}

export function LatencyMonitor({
  latency,
  avg,
  max,
  count,
}: Readonly<LatencyMonitorProps>): React.JSX.Element {
  const getStatusColor = (): string => {
    if (avg < 5) return 'text-green-400';
    if (avg < 10) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getStatusText = (): string => {
    if (avg < 5) return '✓ Excellent';
    if (avg < 10) return '⚠ Fair';
    return '✗ Poor';
  };

  return (
    <div className="mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <h3 className="m-0 mb-3 text-base font-semibold text-gray-200">IPC Latency</h3>
      <div className="grid grid-cols-2 gap-2">
        <div className="flex justify-between rounded bg-plugin-dark p-2">
          <span className="text-sm text-gray-500">Current:</span>
          <span className="font-mono text-sm font-semibold text-accent">
            {latency === null ? '—' : `${latency.toFixed(2)} ms`}
          </span>
        </div>
        <div className="flex justify-between rounded bg-plugin-dark p-2">
          <span className="text-sm text-gray-500">Average:</span>
          <span className="font-mono text-sm font-semibold text-accent">
            {avg > 0 ? `${avg.toFixed(2)} ms` : '—'}
          </span>
        </div>
        <div className="flex justify-between rounded bg-plugin-dark p-2">
          <span className="text-sm text-gray-500">Max:</span>
          <span className="font-mono text-sm font-semibold text-accent">
            {max > 0 ? `${max.toFixed(2)} ms` : '—'}
          </span>
        </div>
        <div className="flex justify-between rounded bg-plugin-dark p-2">
          <span className="text-sm text-gray-500">Samples:</span>
          <span className="font-mono text-sm font-semibold text-accent">{count}</span>
        </div>
      </div>
      <div className="mt-3 text-center text-sm font-semibold">
        {avg > 0 && <span className={getStatusColor()}>{getStatusText()}</span>}
      </div>
    </div>
  );
}
