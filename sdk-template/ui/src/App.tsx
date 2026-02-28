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
    type SignalChainEntry,
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

  // Unified signal chain entries — processors and taps in their declared DSL order.
  // Each processor entry's id must match the snake_case name from `processors: [...]`.
  // Each tap entry's id must match the snake_case name from `taps: [...]`.
  const slotEntries = useMemo<SignalChainEntry[]>(
    () => [
      // processors
      { id: 'test_tone', type: 'processor', component: <TestToneProcessor /> },
      {
        id: 'input_trim',
        type: 'processor',
        component: (
          <GainProcessor processorId="input_trim" title="Input Trim" subtitle="My Input Trim" />
        ),
      },
      {
        id: 'passthrough',
        type: 'processor',
        component: <PassthroughProcessor processorId="passthrough" title="Passthrough" />,
      },
      // slot — example_processor (replace null with your custom processor component)
      // { id: 'example_processor', type: 'processor', component: null },
      { id: 'tone_filter', type: 'processor', component: <ToneFilterProcessor /> },
      { id: 'soft_clip', type: 'processor', component: <SaturatorProcessor /> },
      {
        id: 'output_gain',
        type: 'processor',
        component: (
          <GainProcessor processorId="output_gain" title="Output Gain" subtitle="My Output Gain" />
        ),
      },
      // taps
      { id: 'oscilloscope_tap', type: 'tap', component: <OscilloscopeProcessor /> },
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
      <SignalChain entries={slotEntries} />

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
