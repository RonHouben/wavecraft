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
  OscillatorControl,
} from '@wavecraft/components';

const DEDICATED_PARAMETER_IDS = new Set(['oscillator_enabled']);

export function App(): JSX.Element {
  const { params, isLoading } = useAllParameters();
  const genericParams = params.filter((param) => !DEDICATED_PARAMETER_IDS.has(param.id));
  const groups = useParameterGroups(genericParams);
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

          <OscillatorControl
            isProducing={isOscillatorProducing}
            isOn={isOscillatorOn}
            onToggle={() => {
              void setOscillatorEnabled(isOscillatorOn ? 0 : 1);
            }}
          />

          {isLoading ? (
            <p className="italic text-gray-500">Loading parameters...</p>
          ) : (
            <div className="space-y-4">
              {groups.length > 0
                ? groups.map((group) => <ParameterGroup key={group.name} group={group} />)
                : genericParams.map((p) => <ParameterSlider key={p.id} id={p.id} />)}
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
