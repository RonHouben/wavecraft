import {
  GainProcessor,
  Header,
  LatencyMonitor,
  Meter,
  OscilloscopeProcessor,
  PassthroughProcessor,
  ResizeHandle,
  SaturatorProcessor,
  Settings,
  SignalChain,
  ToneFilterProcessor,
} from '@wavecraft/components';
import { TestToneProcessor } from '@wavecraft/components/processors/TestToneProcessor';
import {
  useConnectionStatus,
  useLatencyMonitor,
  useMeterFrame,
  useRequestResize,
  useWindowResizeSync,
  WavecraftProvider,
} from '@wavecraft/core';
import { type JSX } from 'react';

export function App(): JSX.Element {
  useWindowResizeSync();
  const { connected } = useConnectionStatus();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

  return (
    <WavecraftProvider>
      <Header title="My Cool Plugin">
        <Settings />
      </Header>
      <SignalChain
        entries={[
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
              <GainProcessor
                processorId="output_gain"
                title="Output Gain"
                subtitle="My Output Gain"
              />
            ),
          },
          // taps
          { id: 'OscilloscopeTap', type: 'tap', component: <OscilloscopeProcessor /> },
        ]}
      />

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
