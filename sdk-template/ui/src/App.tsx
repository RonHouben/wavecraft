import {
  GainProcessor,
  Header,
  IconButton,
  LatencyMonitor,
  Meter,
  OscilloscopeProcessor,
  PassthroughProcessor,
  ResizeHandle,
  SaturatorProcessor,
  SettingsModal,
  Sidebar,
  SignalChain,
  ToneFilterProcessor,
} from '@wavecraft/components';
import { Button } from '@wavecraft/components/Button';
import { TestToneProcessor } from '@wavecraft/components/processors/TestToneProcessor';
import { useWindowResizeSync, WavecraftProvider } from '@wavecraft/core';
import { type JSX, useState } from 'react';

export function App(): JSX.Element {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  useWindowResizeSync();

  return (
    <WavecraftProvider>
      {({ openSettingsModal }) => (
        <>
          <div className="flex flex-col gap-3 px-3 pb-16 pr-16 pt-3">
            <Header title="My Cool Plugin">
              {!isSidebarOpen && (
                <IconButton
                  icon="menu"
                  size="sm"
                  className="bg-plugin-surface-1/80"
                  onClick={() => setIsSidebarOpen(true)}
                />
              )}
            </Header>

            <Sidebar
              id="app-sidebar"
              open={isSidebarOpen}
              onClose={() => setIsSidebarOpen(false)}
              title="Menu"
              description="Quick actions and future plugin settings live here."
              defaultActions={['show-settings']}
            >
              <Button
                className="w-full justify-between bg-plugin-surface-1 text-plugin-text-primary hover:bg-plugin-surface-2"
                size="md"
                iconLeft="settings"
                iconRight="chevron-right"
                onClick={openSettingsModal}
              >
                Settings
              </Button>
            </Sidebar>

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
              <Meter className="justify-center" />
              <LatencyMonitor className="justify-center" />
            </div>
          </div>

          <SettingsModal />
          <ResizeHandle />
        </>
      )}
    </WavecraftProvider>
  );
}
