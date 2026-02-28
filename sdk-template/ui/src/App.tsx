import {
  ConnectionStatus,
  GainProcessor,
  LatencyMonitor,
  Meter,
  OscilloscopeProcessor,
  PassthroughProcessor,
  ResizeHandle,
  SaturatorProcessor,
  SignalChain,
  TestToneProcessor,
  ToneFilterProcessor,
  VersionBadge,
  type SignalChainProcessorEntry,
} from '@wavecraft/components';
import {
  useAudioStatus,
  useConnectionStatus,
  useLatencyMonitor,
  useMeterFrame,
  useRequestResize,
  useWindowResizeSync,
  WavecraftProvider,
} from '@wavecraft/core';
import { useMemo, type JSX } from 'react';

export function App(): JSX.Element {
  useWindowResizeSync();
  const { connected, transport } = useConnectionStatus();
  const { phase, isReady, isDegraded, diagnostic } = useAudioStatus();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

  // Processor entries in Rust registration slot order.
  // Each entry's array index IS its slot index in the engine (must match
  // the `processors: [...]` list in engine/src/lib.rs).
  const processorEntries = useMemo<SignalChainProcessorEntry[]>(
    () => [
      // slot 0 — test_tone
      { id: 'test_tone', component: <TestToneProcessor /> },
      // slot 1 — input_trim
      {
        id: 'input_trim',
        component: (
          <GainProcessor processorId="input_trim" title="Input Trim" subtitle="My Input Trim" />
        ),
      },
      // slot 2 — passthrough
      {
        id: 'passthrough',
        component: <PassthroughProcessor processorId="passthrough" title="Passthrough" />,
      },
      // slot 3 — example_processor (replace null with your custom processor component)
      // { id: 'example_processor', component: null },
      // slot 4 — tone_filter
      { id: 'tone_filter', component: <ToneFilterProcessor /> },
      // slot 5 — soft_clip
      { id: 'soft_clip', component: <SaturatorProcessor /> },
      // slot 6 — output_gain
      {
        id: 'output_gain',
        component: (
          <GainProcessor processorId="output_gain" title="Output Gain" subtitle="My Output Gain" />
        ),
      },
      // slot 7 — oscilloscope_tap
      { id: 'oscilloscope_tap', component: <OscilloscopeProcessor /> },
    ],
    []
  );

  return (
    <WavecraftProvider>
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

      {/* Signal chain — drag-and-drop reorderable */}
      <SignalChain processors={processorEntries} />

      {/* Monitoring */}
      <div className="flex flex-col gap-2">
        <Meter className="justify-center" connected={connected} frame={frame} />
        <LatencyMonitor
          className="justify-center"
          latency={latency.latency}
          avg={latency.avg}
          max={latency.max}
          count={latency.count}
        />
      </div>

      <ResizeHandle onRequestResize={requestResize} />
    </WavecraftProvider>
  );
}
