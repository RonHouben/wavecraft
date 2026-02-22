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

import { SmartProcessor } from '../../../../sdk-template/ui/src/processors/SmartProcessor';

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

describe('sdk-template SmartProcessor primitive migration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSetParameter.mockImplementation(async () => undefined);
  });

  it('renders primitive controls and forwards parameter changes', () => {
    const params: ParameterInfo[] = [
      makeParameter({
        id: 'oscillator_waveform',
        name: 'Waveform',
        type: 'enum',
        value: 0,
        variants: ['Sine', 'Square'],
      }),
      makeParameter({
        id: 'oscillator_frequency',
        name: 'Frequency',
        type: 'float',
        unit: 'Hz',
        value: 440,
      }),
      makeParameter({ id: 'oscillator_bypass', name: 'Bypass', type: 'bool', value: true }),
      makeParameter({
        id: 'oscillator_level',
        name: 'Output Level',
        type: 'float',
        unit: 'dB',
        value: -6,
        min: -24,
        max: 0,
      }),
    ];

    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParametersForProcessor.mockReturnValue({
      params,
      isLoading: false,
      error: null,
      setParameter: mockSetParameter,
    });

    render(<SmartProcessor id="oscillator" title="Oscillator" />);

    const controlsContainer = document.querySelector('section .relative.space-y-3');
    expect(controlsContainer?.firstElementChild).toHaveTextContent('Bypass');

    fireEvent.click(screen.getByRole('switch', { name: 'Bypass' }));
    fireEvent.click(screen.getByRole('button', { name: 'Square' }));

    fireEvent.change(screen.getByLabelText('Frequency'), {
      target: { value: '880' },
    });
    fireEvent.change(screen.getByLabelText('Output Level'), {
      target: { value: '-3' },
    });

    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_bypass', false);
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

    const { rerender } = render(<SmartProcessor id="oscillator" title="Oscillator" />);

    const loadingState = screen.getByRole('status');
    expect(loadingState).toHaveTextContent('Loading oscillator...');
    expect(loadingState).toHaveAttribute('aria-live', 'polite');

    mockUseParametersForProcessor.mockReturnValue({
      params: [],
      isLoading: false,
      error: new Error('boom'),
      setParameter: mockSetParameter,
    });

    rerender(<SmartProcessor id="oscillator" title="Oscillator" />);

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

    render(<SmartProcessor id="oscillator" title="Oscillator" />);

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

    const { container } = render(
      <SmartProcessor id="oscillator" title="Oscillator" hideWhenNotInSignalChain />
    );

    expect(container.firstChild).toBeNull();
  });
});
