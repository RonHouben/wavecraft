import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import type { ParameterInfo } from '@wavecraft/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Source under test lives in sdk-template/ui/src/processors/OscillatorProcessor.tsx.

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

function mockSliderRect(input: HTMLElement, top: number, height: number) {
  return vi.spyOn(input, 'getBoundingClientRect').mockReturnValue({
    x: 0,
    y: top,
    top,
    left: 0,
    right: 20,
    bottom: top + height,
    width: 20,
    height,
    toJSON: (): Record<string, never> => ({}),
  } as DOMRect);
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
    expect(screen.getByRole('heading', { name: 'Oscillator' }).closest('section')).toHaveAttribute(
      'data-bypassed',
      'true'
    );
    expect(screen.getByRole('heading', { name: 'Oscillator' }).closest('section')).toHaveClass(
      'opacity-70',
      'saturate-50',
      'brightness-90'
    );
    expect(screen.getByRole('button', { name: 'Bypass' })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('button', { name: 'Enabled' })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('slider', { name: 'Level' })).toHaveAttribute(
      'aria-orientation',
      'vertical'
    );
    expect(screen.getByRole('slider', { name: 'Level' })).toHaveClass('[writing-mode:vertical-lr]');

    fireEvent.click(screen.getByRole('button', { name: 'Bypass' }));
    fireEvent.click(screen.getByRole('button', { name: 'Enabled' }));
    fireEvent.click(screen.getByRole('button', { name: 'Square' }));

    fireEvent.change(screen.getByLabelText('Frequency'), {
      target: { value: '880' },
    });
    const levelSlider = screen.getByLabelText('Level');
    const rectSpy = mockSliderRect(levelSlider, 100, 160);
    fireEvent.pointerDown(levelSlider, { pointerId: 11, clientY: 180, shiftKey: false });
    fireEvent.pointerMove(levelSlider, { pointerId: 11, clientY: 100, shiftKey: false });
    fireEvent.pointerUp(levelSlider, { pointerId: 11, clientY: 100 });
    rectSpy.mockRestore();

    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_bypass', false);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_enabled', false);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_waveform', 1);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_frequency', 880);
    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_level', 0);
  });

  it('keeps oscillator level vertical when other level parameters are present', () => {
    const params: ParameterInfo[] = [
      makeParameter({
        id: 'preamp_level',
        name: 'Preamp Level',
        type: 'float',
        value: -9,
        min: -24,
        max: 0,
        unit: 'dB',
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

    const { container } = render(<OscillatorProcessor />);

    const oscillatorLevelControl =
      container.querySelector<HTMLInputElement>('#param-oscillator_level');
    const preampLevelControl = container.querySelector<HTMLInputElement>('#param-preamp_level');

    expect(oscillatorLevelControl).not.toBeNull();
    expect(preampLevelControl).not.toBeNull();
    expect(oscillatorLevelControl).toHaveAttribute('aria-orientation', 'vertical');
    expect(preampLevelControl).toHaveAttribute('aria-orientation', 'horizontal');
  });

  it('keeps level percent value in a stable-width numeric slot to avoid layout jitter', () => {
    const params: ParameterInfo[] = [
      makeParameter({ id: 'oscillator_frequency', name: 'Frequency', type: 'float', value: 440 }),
      makeParameter({
        id: 'oscillator_level',
        name: 'Level',
        type: 'float',
        value: 1,
        min: 0,
        max: 1,
        unit: '%',
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

    const levelValue = screen.getByText('100.0%');
    expect(levelValue).toHaveClass('min-w-[6ch]');
    expect(levelValue).toHaveClass('text-right');
  });

  it('shows a global precision hint for focused controls and switches hint text while Shift precision is active', async () => {
    const params: ParameterInfo[] = [
      makeParameter({ id: 'oscillator_bypass', name: 'Bypass', type: 'bool', value: true }),
      makeParameter({ id: 'oscillator_enabled', name: 'Enabled', type: 'bool', value: true }),
      makeParameter({ id: 'oscillator_frequency', name: 'Frequency', type: 'float', value: 440 }),
    ];

    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParametersForProcessor.mockReturnValue({
      params,
      isLoading: false,
      error: null,
      setParameter: mockSetParameter,
    });

    render(<OscillatorProcessor />);

    const hint = screen.getByTestId('processor-precision-hint');
    const frequencyInput = screen.getByLabelText('Frequency');

    expect(hint).toHaveTextContent('');

    fireEvent.focus(frequencyInput);
    expect(hint).toHaveTextContent('Hold Shift for fine adjust');

    fireEvent.keyDown(frequencyInput, { key: 'Shift' });
    expect(hint).toHaveTextContent('⇧ Fine adjust');

    fireEvent.keyUp(frequencyInput, { key: 'Shift' });
    expect(hint).toHaveTextContent('Hold Shift for fine adjust');

    fireEvent.blur(frequencyInput);

    await waitFor(() => {
      expect(hint).toHaveTextContent('');
    });
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

    fireEvent.click(screen.getByRole('button', { name: 'Bypass' }));

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
