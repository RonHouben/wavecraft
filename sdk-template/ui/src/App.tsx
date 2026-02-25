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
      <Col className="grid grid-cols-12 gap-4 px-4">
        <Row className="col-span-12 grid grid-cols-12 justify-center gap-4">
          <TestToneProcessor className="col-span-3" hideWhenNotInSignalChain />
          <GainProcessor
            className="col-span-3"
            processorId="input_trim"
            title="Input Trim"
            subtitle="My Input Trim"
            hideWhenNotInSignalChain
          />
          <PassthroughProcessor
            processorId="passthrough"
            className="col-span-3"
            hideWhenNotInSignalChain
            title="Passthrough"
          />
          <GainProcessor
            className="col-span-3"
            processorId="output_gain"
            title="Output Gain"
            subtitle="My Output Gain"
            hideWhenNotInSignalChain
          />
        </Row>
        <Row className="col-span-12 grid grid-cols-12">
          <OscilloscopeProcessor className="col-span-12" hideWhenNotInSignalChain />
          {/* <ExampleProcessor hideWhenNotInSignalChain /> */}
          {/* <SmartProcessor id="output_gain" title="Output Gain" hideWhenNotInSignalChain /> */}
          {/* <OscilloscopeProcessor hideWhenNotInSignalChain /> */}
        </Row>
        <Row className="col-span-12 grid grid-cols-12 justify-center gap-4">
          {/* Metering Section */}
          <Meter className="col-span-6 h-full justify-center" connected={connected} frame={frame} />

          <LatencyMonitor
            className="col-span-6"
            latency={latency.latency}
            avg={latency.avg}
            max={latency.max}
            count={latency.count}
          />
        </Row>
      </Col>

      <ResizeHandle onRequestResize={requestResize} />
    </WavecraftProvider>
  );
}
