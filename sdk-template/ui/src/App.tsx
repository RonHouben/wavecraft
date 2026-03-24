import {
  ConnectionStatus,
  GainProcessor,
  LatencyMonitor,
  Meter,
  OscilloscopeProcessor,
  PassthroughProcessor,
  ResizeHandle,
  SaturatorProcessor,
  Select,
  SignalChain,
  SignalChainOrderDebugPanel,
  TestToneProcessor,
  ToneFilterProcessor,
  VersionBadge,
  type SignalChainEntry,
} from '@wavecraft/components';
import {
  useAudioStatus,
  useConnectionStatus,
  useHardwareInputSelection,
  useInputSource,
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
  const {
    selected: selectedInputSource,
    available: availableInputSources,
    setSelected: setSelectedInputSource,
    isLoading: isInputSourceLoading,
  } = useInputSource();
  const {
    selectedDevice,
    selectedChannelId,
    availableChannels,
    setSelectedChannel,
    isLoading: isHardwareInputSelectionLoading,
  } = useHardwareInputSelection();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

  // Unified signal chain entries — processors and taps in their declared runtime order.
  // These IDs must match the backend/runtime `SignalChainSlot.id` values so the
  // drag-and-drop presentation can reconcile UI cards with server-authoritative order.
  const slotEntries = useMemo<SignalChainEntry[]>(
    () => [
      // processors
      { id: 'TestTone', type: 'processor', component: <TestToneProcessor /> },
      {
        id: 'InputTrim',
        type: 'processor',
        component: (
          <GainProcessor processorId="input_trim" title="Input Trim" subtitle="My Input Trim" />
        ),
      },
      {
        id: 'Passthrough',
        type: 'processor',
        component: <PassthroughProcessor processorId="passthrough" title="Passthrough" />,
      },
      { id: 'ToneFilter', type: 'processor', component: <ToneFilterProcessor /> },
      { id: 'SoftClip', type: 'processor', component: <SaturatorProcessor /> },
      {
        id: 'OutputGain',
        type: 'processor',
        component: (
          <GainProcessor processorId="output_gain" title="Output Gain" subtitle="My Output Gain" />
        ),
      },
      // taps
      { id: 'OscilloscopeTap', type: 'tap', component: <OscilloscopeProcessor /> },
    ],
    []
  );

  return (
    <WavecraftProvider>
      {/* Header */}
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-100">My Plugin</h1>
          <div className="flex items-center gap-2">
            {selectedInputSource ? (
              <Select
                label="Input Source"
                size="sm"
                value={selectedInputSource}
                options={availableInputSources.map((source) => ({
                  label: source.label,
                  value: source.id,
                }))}
                disabled={!connected || availableInputSources.length === 0}
                state={isInputSourceLoading ? 'loading' : 'default'}
                onChange={(nextValue) => {
                  void setSelectedInputSource(nextValue);
                }}
              />
            ) : null}
            {selectedDevice ? (
              <div className="flex items-center gap-2 rounded-md border border-plugin-border bg-plugin-surface px-2.5 py-1.5 text-type-xs text-plugin-text-secondary shadow-control">
                <span className="font-medium text-plugin-text-primary">Device</span>
                <span>{selectedDevice.label}</span>
              </div>
            ) : null}
            {selectedChannelId ? (
              <Select
                label="Input Channels"
                size="sm"
                value={selectedChannelId}
                options={availableChannels.map((channel) => ({
                  label: channel.label,
                  value: channel.id,
                }))}
                disabled={
                  !connected ||
                  selectedInputSource !== 'hardwareInput' ||
                  availableChannels.length === 0
                }
                state={isHardwareInputSelectionLoading ? 'loading' : 'default'}
                onChange={(nextValue) => {
                  void setSelectedChannel(nextValue);
                }}
              />
            ) : null}
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
        {selectedInputSource === 'testTone' ? (
          <p
            data-testid="input-source-dev-note"
            className="text-type-xs text-plugin-text-secondary"
          >
            Browser dev input feeds the full chain. Processor cards do not show stage-local signal
            activity.
          </p>
        ) : null}
      </div>

      <SignalChainOrderDebugPanel />

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
