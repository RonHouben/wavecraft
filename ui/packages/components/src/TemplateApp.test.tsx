import { render, screen } from '@testing-library/react';
import type { ParameterInfo } from '@wavecraft/core';
import { describe, expect, it, vi } from 'vitest';

const mockUseWindowResizeSync = vi.hoisted(() => vi.fn());
const mockUseAllParameters = vi.hoisted(() => vi.fn());
const mockUseParameterGroups = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useWindowResizeSync: mockUseWindowResizeSync,
  useAllParameters: mockUseAllParameters,
  useParameterGroups: mockUseParameterGroups,
}));

vi.mock('@wavecraft/components', () => ({
  Meter: () => <div data-testid="meter" />,
  VersionBadge: () => <div data-testid="version-badge" />,
  ConnectionStatus: () => <div data-testid="connection-status" />,
  LatencyMonitor: () => <div data-testid="latency-monitor" />,
  OscillatorControl: () => <div data-testid="oscillator-control" />,
  Oscilloscope: () => <div data-testid="oscilloscope" />,
  ParameterGroup: ({ group }: { group: { name: string } }) => (
    <div data-testid={`group-${group.name}`} />
  ),
  ResizeHandle: () => <div data-testid="resize-handle" />,
}));

import { App } from '../../../../sdk-template/ui/src/App';

describe('sdk-template App layout', () => {
  it('renders oscillator panel and resize handle', () => {
    const params = [
      { id: 'oscillator_enabled' },
      { id: 'tone_filter_cutoff_hz' },
    ] as ParameterInfo[];

    mockUseAllParameters.mockReturnValue({
      params,
      isLoading: false,
      error: null,
      reload: vi.fn(),
    });
    mockUseParameterGroups.mockReturnValue([
      {
        name: 'Filter',
        parameters: [params[1]],
      },
    ]);

    render(<App />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
    expect(screen.getByTestId('group-Filter')).toBeInTheDocument();
    expect(screen.getByTestId('resize-handle')).toBeInTheDocument();
  });
});
