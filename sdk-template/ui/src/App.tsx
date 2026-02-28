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
import {
  Meter,
  VersionBadge,
  ConnectionStatus,
  LatencyMonitor,
  ResizeHandle,
  TestToneProcessor,
  OscilloscopeProcessor,
  Row,
  Col,
  GainProcessor,
  PassthroughProcessor,
  SaturatorProcessor,
  ToneFilterProcessor,
} from '@wavecraft/components';

export function App(): JSX.Element {
  useWindowResizeSync();
  const { connected, transport } = useConnectionStatus();
  const { phase, isReady, isDegraded, diagnostic } = useAudioStatus();
  const latency = useLatencyMonitor(1000);
  const frame = useMeterFrame(50);
  const requestResize = useRequestResize();

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

      {/* Main Content */}
      <Col className="gap-2 bg-purple-500 px-4 sm:bg-red-500 md:bg-amber-500 lg:bg-green-500">
        <Row className="gap-2">
          <TestToneProcessor
            className="col-span-full sm:col-span-8 md:col-span-4"
            hideWhenNotInSignalChain
          />
          <Col className="col-span-8 gap-2 sm:col-span-4 md:col-span-4">
            <GainProcessor
              className="col-span-full"
              processorId="input_trim"
              title="Input Trim"
              subtitle="My Input Trim"
              hideWhenNotInSignalChain
            />
            <GainProcessor
              className="col-span-full"
              processorId="output_gain"
              title="Output Gain"
              subtitle="My Output Gain"
              hideWhenNotInSignalChain
            />
          </Col>
          <PassthroughProcessor
            processorId="passthrough"
            className="col-span-4 sm:col-span-12 md:col-span-4"
            hideWhenNotInSignalChain
            title="Passthrough"
          />
          <SaturatorProcessor
            className="col-span-full sm:col-span-full md:col-span-6"
            hideWhenNotInSignalChain
          />
          <ToneFilterProcessor
            className="col-span-full sm:col-span-full md:col-span-6"
            hideWhenNotInSignalChain
          />
        </Row>
        <Row className="col-span-full gap-2">
          <OscilloscopeProcessor className="col-span-full md:col-span-6" hideWhenNotInSignalChain />
          <Col className="col-span-full gap-2 md:col-span-6">
            <Meter className="col-span-full justify-center" connected={connected} frame={frame} />
            <LatencyMonitor
              className="col-span-full justify-center"
              latency={latency.latency}
              avg={latency.avg}
              max={latency.max}
              count={latency.count}
            />
          </Col>
        </Row>
      </Col>

      <ResizeHandle onRequestResize={requestResize} />
    </WavecraftProvider>
  );
}
