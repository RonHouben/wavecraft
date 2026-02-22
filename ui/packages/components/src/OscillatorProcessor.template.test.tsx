import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import type { ParameterInfo } from '@wavecraft/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseHasProcessorInSignalChain = vi.hoisted(() => vi.fn());
const mockUseParametersForProcessor = vi.hoisted(() => vi.fn());
const mockSetParameter = vi.hoisted(() => vi.fn(async () => undefined));
const mockLoggerError = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');
  return {
    ...actual,
    logger: {
      error: mockLoggerError,
    },
    useHasProcessorInSignalChain: mockUseHasProcessorInSignalChain,
    useParametersForProcessor: mockUseParametersForProcessor,
  };
});

import { OscillatorProcessor } from '../../../../sdk-template/ui/src/processors/OscillatorProcessor';

function makeParameter(overrides: Partial<ParameterInfo>): ParameterInfo {
  return {
    id: 'oscillator_frequency',
    name: 'Frequency',
    type: 'float',
    value: 440,
    default: 440,
    min: 20,
    max: 20_000,
    unit: 'Hz',
    ...overrides,
  };
}

describe('sdk-template OscillatorProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSetParameter.mockImplementation(async () => undefined);
  });

  it('renders compact grouped controls and forwards parameter changes', () => {
    const params: ParameterInfo[] = [
      makeParameter({ id: 'oscillator_bypass', name: 'Bypass', type: 'bool', value: true }),
      makeParameter({ id: 'oscillator_enabled', name: 'Enabled', type: 'bool', value: true }),
      makeParameter({
        id: 'oscillator_waveform',
        name: 'Waveform',
        type: 'enum',
        value: 0,
        variants: ['Sine', 'Square'],
      }),
      makeParameter({ id: 'oscillator_frequency', name: 'Frequency', type: 'float', value: 440 }),
      makeParameter({
        id: 'oscillator_level',
        name: 'Level',
        type: 'float',
        value: -6,
        min: -24,
        max: 0,
        unit: 'dB',
      }),
    ];

    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParametersForProcessor.mockReturnValue({
      params,
      isLoading: false,
      error: null,
      setParameter: mockSetParameter,
    });

    render(<OscillatorProcessor />);

    expect(screen.getByRole('heading', { name: 'Oscillator' })).toBeInTheDocument();
    expect(screen.getByRole('switch', { name: 'Bypass' })).toBeInTheDocument();
    expect(screen.getByRole('switch', { name: 'Enabled' })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('switch', { name: 'Bypass' }));
    fireEvent.click(screen.getByRole('switch', { name: 'Enabled' }));
    fireEvent.click(screen.getByRole('button', { name: 'Square' }));

    fireEvent.change(screen.getByLabelText('Frequency'), {
      target: { value: '880' },
    });
    fireEvent.change(screen.getByLabelText('Level'), {
      target: { value: '-3' },
    });

    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_bypass', false);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_enabled', false);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_waveform', 1);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_frequency', 880);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_level', -3);
  });

  it('announces loading and error states with live-region semantics', () => {
    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParametersForProcessor.mockReturnValue({
      params: [],
      isLoading: true,
      error: null,
      setParameter: mockSetParameter,
    });

    const { rerender } = render(<OscillatorProcessor />);

    const loadingState = screen.getByRole('status');
    expect(loadingState).toHaveTextContent('Loading oscillator...');
    expect(loadingState).toHaveAttribute('aria-live', 'polite');

    mockUseParametersForProcessor.mockReturnValue({
      params: [],
      isLoading: false,
      error: new Error('boom'),
      setParameter: mockSetParameter,
    });

    rerender(<OscillatorProcessor />);

    const errorState = screen.getByRole('alert');
    expect(errorState).toHaveTextContent('Error loading oscillator: boom');
    expect(errorState).toHaveAttribute('aria-live', 'assertive');
  });

  it('logs when setParameter fails', async () => {
    const failure = new Error('write failed');
    mockSetParameter.mockRejectedValueOnce(failure);

    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParametersForProcessor.mockReturnValue({
      params: [
        makeParameter({ id: 'oscillator_bypass', name: 'Bypass', type: 'bool', value: true }),
      ],
      isLoading: false,
      error: null,
      setParameter: mockSetParameter,
    });

    render(<OscillatorProcessor />);

    fireEvent.click(screen.getByRole('switch', { name: 'Bypass' }));

    await waitFor(() => {
      expect(mockLoggerError).toHaveBeenCalledWith('Failed to set processor parameter', {
        error: failure,
        parameterId: 'oscillator_bypass',
        processorId: 'oscillator',
      });
    });
  });

  it('honors hideWhenNotInSignalChain', () => {
    mockUseHasProcessorInSignalChain.mockReturnValue(false);
    mockUseParametersForProcessor.mockReturnValue({
      params: [],
      isLoading: false,
      error: null,
      setParameter: mockSetParameter,
    });

    const { container } = render(<OscillatorProcessor hideWhenNotInSignalChain />);

    expect(container.firstChild).toBeNull();
  });
});
