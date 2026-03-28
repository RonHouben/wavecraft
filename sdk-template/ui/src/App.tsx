import {
  Button,
  GainProcessor,
  Header,
  IconButton,
  LatencyMonitor,
  Meter,
  OscilloscopeProcessor,
  PassthroughProcessor,
  ResizeHandle,
  SaturatorProcessor,
  Sidebar,
  SignalChain,
  SignalChainOrderDebugPanel,
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
import { type JSX, useCallback, useState } from 'react';

export function App(): JSX.Element {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  const handleToggleSidebar = useCallback(() => {
    setIsSidebarOpen((isOpen) => !isOpen);
  }, []);

  const handleCloseSidebar = useCallback(() => {
    setIsSidebarOpen(false);
  }, []);

  useWindowResizeSync();
  const { connected } = useConnectionStatus();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

  return (
    <WavecraftProvider>
      <div className="flex flex-col gap-3 pb-16 pr-16">
        <Header title="My Cool Plugin">
          <IconButton
            icon="menu"
            aria-label={isSidebarOpen ? 'Close sidebar' : 'Open sidebar'}
            active={isSidebarOpen}
            onClick={handleToggleSidebar}
          />
        </Header>

        <Sidebar open={isSidebarOpen} onClose={handleCloseSidebar} title="Menu">
          <Button size="sm">Settings</Button>
        </Sidebar>

        <SignalChainOrderDebugPanel />

        <SignalChain
          entries={[
            // processors
            { id: 'TestTone', type: 'processor', component: <TestToneProcessor /> },
            {
              id: 'InputTrim',
              type: 'processor',
              component: (
                <GainProcessor
                  processorId="input_trim"
                  title="Input Trim"
                  subtitle="My Input Trim"
                />
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

        <div className="flex flex-col gap-3">
          <Meter className="justify-center" connected={connected} frame={frame} />
          <LatencyMonitor
            className="justify-center"
            latency={latency.latency}
            avg={latency.avg}
            max={latency.max}
            count={latency.count}
          />
        </div>
      </div>

      <ResizeHandle onRequestResize={requestResize} />
    </WavecraftProvider>
  );
}
