import {
  useAudioStatus,
  useConnectionStatus,
  useLatencyMonitor,
  useMeterFrame,
  useRequestResize,
  WavecraftProvider,
  useWindowResizeSync,
} from '@wavecraft/core';
import { type JSX } from 'react';
import { ExampleProcessor } from './processors/ExampleProcessor';
import { SmartProcessor } from './processors/SmartProcessor';
import {
  Meter,
  VersionBadge,
  ConnectionStatus,
  LatencyMonitor,
  ResizeHandle,
} from '@wavecraft/components';
import { OscilloscopeProcessor } from './processors/OscilloscopeProcessor';

export function App(): JSX.Element {
  useWindowResizeSync();
  const { connected, transport } = useConnectionStatus();
  const { phase, isReady, isDegraded, diagnostic } = useAudioStatus();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

  return (
    <WavecraftProvider>
      <div className="flex h-screen flex-col gap-4 bg-plugin-dark p-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-100">My Plugin</h1>
          <div className="flex items-center gap-2">
            <ConnectionStatus
              connected={connected}
              transport={transport}
              phase={phase}
              isReady={isReady}
              isDegraded={isDegraded}
              diagnostic={diagnostic}
            />
            <VersionBadge />
          </div>
        </div>

        {/* Main Content */}
        <div className="flex flex-1 flex-col gap-6">
          <div className="grid grid-cols-1 gap-4">
            <SmartProcessor id="oscillator" title="Oscillator" hideWhenNotInSignalChain />
            <SmartProcessor id="input_trim" title="Input Trim" hideWhenNotInSignalChain />
            <SmartProcessor id="tone_filter" title="Tone Filter" hideWhenNotInSignalChain />
            <SmartProcessor id="soft_clip" title="Soft Clip" hideWhenNotInSignalChain />
            <ExampleProcessor hideWhenNotInSignalChain />
            <SmartProcessor id="output_gain" title="Output Gain" hideWhenNotInSignalChain />
            <OscilloscopeProcessor hideWhenNotInSignalChain />
          </div>

          {/* Metering Section */}
          <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
            <h2 className="mb-3 text-base font-semibold text-gray-200">Output Metering</h2>
            <Meter connected={connected} frame={frame} />
          </div>

          {/* Info Section */}
          <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
            <h2 className="mb-3 text-base font-semibold text-gray-200">Info</h2>
            <LatencyMonitor
              latency={latency.latency}
              avg={latency.avg}
              max={latency.max}
              count={latency.count}
            />
          </div>
        </div>

        <ResizeHandle onRequestResize={requestResize} />
      </div>
    </WavecraftProvider>
  );
}
