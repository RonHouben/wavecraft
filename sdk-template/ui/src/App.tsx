import {
  useAllParameters,
  useMeterFrame,
  useParameter,
  useParameterGroups,
  useWindowResizeSync,
} from '@wavecraft/core';
import { type JSX } from 'react';
import {
  Meter,
  ParameterSlider,
  ParameterGroup,
  VersionBadge,
  ConnectionStatus,
  LatencyMonitor,
} from '@wavecraft/components';

export function App(): JSX.Element {
  const { params, isLoading } = useAllParameters();
  const groups = useParameterGroups(params);
  const meterFrame = useMeterFrame(100);
  const { param: oscillatorEnabled, setValue: setOscillatorEnabled } =
    useParameter('oscillator_enabled');

  const oscillatorPeak = Math.max(meterFrame?.peak_l ?? 0, meterFrame?.peak_r ?? 0);
  const isOscillatorProducing = oscillatorPeak > 1e-4;
  const isOscillatorOn = (oscillatorEnabled?.value ?? 1) >= 0.5;

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
        {/* Parameters Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">Parameters</h2>

          {oscillatorEnabled && (
            <div className="mb-3 rounded-md border border-plugin-border bg-plugin-dark p-3">
              <div className="mb-2 flex items-center justify-between gap-3">
                <span className="text-sm font-semibold text-gray-200">Oscillator signal</span>
                <span
                  className={`rounded px-2 py-1 text-xs font-semibold ${
                    isOscillatorProducing
                      ? 'bg-green-900/30 text-green-400'
                      : 'bg-yellow-900/30 text-yellow-400'
                  }`}
                >
                  {isOscillatorProducing ? 'Producing' : 'No signal'}
                </span>
              </div>

              <div className="flex items-center justify-between gap-3">
                <span className="text-sm text-gray-300">
                  Oscillator output: {isOscillatorOn ? 'On' : 'Off'}
                </span>
                <button
                  type="button"
                  className={`rounded px-3 py-1.5 text-sm font-semibold ${
                    isOscillatorOn
                      ? 'bg-red-900/30 text-red-300 hover:bg-red-900/45'
                      : 'bg-green-900/30 text-green-300 hover:bg-green-900/45'
                  }`}
                  onClick={() => {
                    void setOscillatorEnabled(isOscillatorOn ? 0 : 1);
                  }}
                >
                  {isOscillatorOn ? 'Turn Off' : 'Turn On'}
                </button>
              </div>
            </div>
          )}

          {isLoading ? (
            <p className="italic text-gray-500">Loading parameters...</p>
          ) : (
            <div className="space-y-4">
              {groups.length > 0
                ? groups.map((group) => <ParameterGroup key={group.name} group={group} />)
                : params?.map((p) => <ParameterSlider key={p.id} id={p.id} />)}
            </div>
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
    </div>
  );
}
