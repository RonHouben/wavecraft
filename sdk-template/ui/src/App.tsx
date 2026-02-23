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
  Button,
  Col,
} from '@wavecraft/components';
import { SmartProcessor } from './processors/SmartProcessor';

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
      <Col>
        <Row>
          <TestToneProcessor hideWhenNotInSignalChain />
          <SmartProcessor
            processorId="input_trim"
            bypassParameterId="input_trim_bypass"
            title="Input Trim"
            hideWhenNotInSignalChain
          />
          <SmartProcessor
            processorId="soft_clip"
            bypassParameterId="soft_clip_bypass"
            title="Soft Clip"
            hideWhenNotInSignalChain
          />
          <SmartProcessor
            processorId="tone_filter"
            bypassParameterId="tone_filter_bypass"
            title="Tone Filter"
            hideWhenNotInSignalChain
            radioGroupOptions={{
              renderOptionsAs: Button,
              size: 'lg',
            }}
          />
        </Row>
        <Row>
          <Col>
            <OscilloscopeProcessor hideWhenNotInSignalChain />
            {/* <ExampleProcessor hideWhenNotInSignalChain /> */}
            {/* <SmartProcessor id="output_gain" title="Output Gain" hideWhenNotInSignalChain /> */}
            {/* <OscilloscopeProcessor hideWhenNotInSignalChain /> */}
          </Col>
        </Row>
      </Col>

      {/* Metering Section */}
      <Meter connected={connected} frame={frame} />

      <LatencyMonitor
        latency={latency.latency}
        avg={latency.avg}
        max={latency.max}
        count={latency.count}
      />

      <ResizeHandle onRequestResize={requestResize} />
    </WavecraftProvider>
  );
}
