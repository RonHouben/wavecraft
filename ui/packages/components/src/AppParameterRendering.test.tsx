import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  ConnectionStatus,
  LatencyMonitor,
  Meter,
  OscillatorControl,
  ParameterGroup,
  ParameterSlider,
  VersionBadge,
} from './index';
import { useAllParameters, useParameterGroups, useWindowResizeSync } from '@wavecraft/core';

interface TestParameter {
  id: string;
  name: string;
  type: 'float' | 'bool' | 'enum';
  value: number | boolean;
  default: number | boolean;
  unit?: string;
  group?: string;
}

const DEDICATED_PARAMETER_IDS = new Set([
  'oscillator_enabled',
  'oscillator_waveform',
  'oscillator_frequency',
  'oscillator_level',
]);

function TestApp(): JSX.Element {
  const { params, isLoading } = useAllParameters();
  const genericParams = params.filter((param) => !DEDICATED_PARAMETER_IDS.has(param.id));
  const groups = useParameterGroups(genericParams);

  useWindowResizeSync();

  return (
    <div>
      <ConnectionStatus />
      <VersionBadge />
      <OscillatorControl />
      {isLoading
        ? null
        : groups.length > 0
          ? groups.map((group) => <ParameterGroup key={group.name} group={group} />)
          : genericParams.map((p) => <ParameterSlider key={p.id} id={p.id} />)}
      <Meter />
      <LatencyMonitor />
    </div>
  );
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
      type: 'bool',
      value: true,
      default: true,
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
      id: 'oscillator_waveform',
      name: 'Waveform',
      type: 'enum',
      value: 0,
      default: 0,
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
    render(<TestApp />);

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

    render(<TestApp />);

    expect(screen.getByTestId('group-Main')).toBeInTheDocument();
    expect(screen.queryByTestId('slider-gain')).not.toBeInTheDocument();

    const lastCall = mockUseParameterGroups.mock.calls[
      mockUseParameterGroups.mock.calls.length - 1
    ] as [TestParameter[]] | undefined;
    const groupedInput = lastCall?.[0];
    expect(groupedInput?.map((param) => param.id)).toEqual(['gain']);
  });
});
