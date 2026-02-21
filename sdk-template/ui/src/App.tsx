import { useAllParameters, useParameterGroups, useWindowResizeSync } from '@wavecraft/core';
import { type JSX } from 'react';
import {
  Meter,
  VersionBadge,
  ConnectionStatus,
  LatencyMonitor,
  OscillatorControl,
  Oscilloscope,
  ParameterGroup,
  ResizeHandle,
} from '@wavecraft/components';

const DEDICATED_PARAMETER_IDS = new Set([
  'oscillator_enabled',
  'oscillator_waveform',
  'oscillator_frequency',
  'oscillator_level',
]);

export function App(): JSX.Element {
  useWindowResizeSync();

  const { params, isLoading } = useAllParameters();
  const genericParams = params.filter((param) => !DEDICATED_PARAMETER_IDS.has(param.id));
  const groups = useParameterGroups(genericParams);

  return (
    <div className="flex h-screen flex-col gap-4 bg-plugin-dark p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-100">My Plugin</h1>
        <div className="flex items-center gap-2">
          <ConnectionStatus />
          <VersionBadge />
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 flex-col gap-6">
        <div className="grid grid-cols-2 gap-4">
          <OscillatorControl hideWhenNotInSignalChain />
          <Oscilloscope hideWhenNotInSignalChain />
        </div>

        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">Processor Controls</h2>
          <p className="mb-3 text-xs text-gray-400">
            Bypass controls are generated automatically per processor as <code>*_bypass</code>{' '}
            parameters.
          </p>
          {isLoading ? (
            <p className="text-sm text-gray-400">Loading parameters…</p>
          ) : groups.length > 0 ? (
            <div className="space-y-4">
              {groups.map((group) => (
                <ParameterGroup key={group.name} group={group} />
              ))}
            </div>
          ) : (
            <p className="text-sm text-gray-400">No generic processor parameters discovered.</p>
          )}
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

      <ResizeHandle />
    </div>
  );
}
