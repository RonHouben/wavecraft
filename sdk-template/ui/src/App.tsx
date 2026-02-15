import { useWindowResizeSync } from '@wavecraft/core';
import { type JSX } from 'react';
import {
  Meter,
  VersionBadge,
  ConnectionStatus,
  LatencyMonitor,
  OscillatorControl,
  Oscilloscope,
} from '@wavecraft/components';

export function App(): JSX.Element {
  useWindowResizeSync();

  return (
    <div className="flex h-screen flex-col gap-4 bg-plugin-dark p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-100">{'{{ plugin_name_title }}'}</h1>
        <div className="flex items-center gap-2">
          <ConnectionStatus />
          <VersionBadge />
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 flex-col gap-6">
        <div className="grid grid-cols-2 gap-4">
          <OscillatorControl />
          <Oscilloscope />
        </div>
        {/* Metering Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">Output Metering</h2>
          <Meter />
        </div>

        {/* Info Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">Info</h2>
          <LatencyMonitor />
        </div>
      </div>
    </div>
  );
}
