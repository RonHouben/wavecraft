import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../../../sdk-template/ui/src/App';

interface TestParameter {
  id: string;
  name: string;
  type: 'float' | 'bool' | 'enum';
  value: number;
  default: number;
  unit?: string;
  group?: string;
}

const mockUseAllParameters = vi.hoisted(() => vi.fn());
const mockUseParameterGroups = vi.hoisted(() => vi.fn());
const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const mockUseWindowResizeSync = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useAllParameters: mockUseAllParameters,
  useParameterGroups: mockUseParameterGroups,
  useMeterFrame: mockUseMeterFrame,
  useWindowResizeSync: mockUseWindowResizeSync,
}));

vi.mock('@wavecraft/components', () => ({
  Meter: () => <div data-testid="meter" />,
  ParameterSlider: ({ id }: { id: string }) => <div data-testid={`slider-${id}`} />,
  ParameterGroup: ({ group }: { group: { name: string } }) => (
    <div data-testid={`group-${group.name}`} />
  ),
  VersionBadge: () => <div data-testid="version-badge" />,
  ConnectionStatus: () => <div data-testid="connection-status" />,
  LatencyMonitor: () => <div data-testid="latency-monitor" />,
  OscillatorControl: () => <div data-testid="oscillator-control" />,
}));

describe('App parameter rendering', () => {
  const baseParams: TestParameter[] = [
    {
      id: 'oscillator_enabled',
      name: 'Enabled',
      type: 'float',
      value: 1,
      default: 1,
      group: 'Oscillator',
    },
    {
      id: 'oscillator_frequency',
      name: 'Frequency',
      type: 'float',
      value: 0.5,
      default: 0.5,
      group: 'Oscillator',
    },
    {
      id: 'oscillator_level',
      name: 'Level',
      type: 'float',
      value: 0.75,
      default: 0.75,
      group: 'Oscillator',
    },
    {
      id: 'gain',
      name: 'Gain',
      type: 'float',
      value: 0.5,
      default: 0.5,
      group: 'Main',
    },
  ];

  beforeEach(() => {
    mockUseAllParameters.mockReturnValue({ params: baseParams, isLoading: false });
    mockUseParameterGroups.mockImplementation(() => []);
    mockUseMeterFrame.mockReturnValue(undefined);
    mockUseWindowResizeSync.mockImplementation(() => undefined);
  });

  it('renders oscillator only through dedicated control, not as generic slider', () => {
    render(<App />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
    expect(screen.getByTestId('slider-gain')).toBeInTheDocument();
    expect(screen.queryByTestId('slider-oscillator_enabled')).not.toBeInTheDocument();
    expect(screen.queryByTestId('slider-oscillator_frequency')).not.toBeInTheDocument();
    expect(screen.queryByTestId('slider-oscillator_level')).not.toBeInTheDocument();

    const lastCall = mockUseParameterGroups.mock.calls[
      mockUseParameterGroups.mock.calls.length - 1
    ] as [TestParameter[]] | undefined;
    const groupedInput = lastCall?.[0];
    expect(groupedInput?.map((param) => param.id)).toEqual(['gain']);
  });

  it('filters oscillator_enabled before grouped rendering as well', () => {
    mockUseParameterGroups.mockImplementation((parameters: TestParameter[]) => [
      { name: 'Main', parameters },
    ]);

    render(<App />);

    expect(screen.getByTestId('group-Main')).toBeInTheDocument();
    expect(screen.queryByTestId('slider-gain')).not.toBeInTheDocument();

    const lastCall = mockUseParameterGroups.mock.calls[
      mockUseParameterGroups.mock.calls.length - 1
    ] as [TestParameter[]] | undefined;
    const groupedInput = lastCall?.[0];
    expect(groupedInput?.map((param) => param.id)).toEqual(['gain']);
  });
});
