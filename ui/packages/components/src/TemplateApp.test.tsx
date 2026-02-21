import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

const mockUseWindowResizeSync = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useWindowResizeSync: mockUseWindowResizeSync,
}));

vi.mock('@wavecraft/components', () => ({
  Meter: () => <div data-testid="meter" />,
  VersionBadge: () => <div data-testid="version-badge" />,
  ConnectionStatus: () => <div data-testid="connection-status" />,
  LatencyMonitor: () => <div data-testid="latency-monitor" />,
  OscillatorProcessor: () => <div data-testid="oscillator-control" />,
  OscilloscopeProcessor: () => <div data-testid="oscilloscope" />,
  InputTrimProcessor: () => <div data-testid="processor-input-trim" />,
  OutputGainProcessor: () => <div data-testid="processor-output-gain" />,
  SoftClipProcessor: () => <div data-testid="processor-soft-clip" />,
  ToneFilterProcessor: () => <div data-testid="processor-tone-filter" />,
  ResizeHandle: () => <div data-testid="resize-handle" />,
}));

vi.mock('../../../../sdk-template/ui/src/processors/ExampleProcessor', () => ({
  ExampleProcessor: () => <div data-testid="processor-example" />,
}));

import { App } from '../../../../sdk-template/ui/src/App';

describe('sdk-template App layout', () => {
  it('renders oscillator panel and resize handle', () => {
    render(<App />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
    expect(screen.getByTestId('processor-input-trim')).toBeInTheDocument();
    expect(screen.getByTestId('processor-tone-filter')).toBeInTheDocument();
    expect(screen.getByTestId('processor-soft-clip')).toBeInTheDocument();
    expect(screen.getByTestId('processor-example')).toBeInTheDocument();
    expect(screen.getByTestId('processor-output-gain')).toBeInTheDocument();
    expect(screen.getByTestId('resize-handle')).toBeInTheDocument();
  });
});
