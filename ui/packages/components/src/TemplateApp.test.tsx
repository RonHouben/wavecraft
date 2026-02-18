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
  OscillatorControl: () => <div data-testid="oscillator-control" />,
  Oscilloscope: () => <div data-testid="oscilloscope" />,
  ResizeHandle: () => <div data-testid="resize-handle" />,
}));

import { App } from '../../../../sdk-template/ui/src/App';

describe('sdk-template App layout', () => {
  it('renders oscillator panel and resize handle', () => {
    render(<App />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
    expect(screen.getByTestId('resize-handle')).toBeInTheDocument();
  });
});
