import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { OscillatorProcessor } from './OscillatorProcessor';

const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const mockUseParameter = vi.hoisted(() => vi.fn());
const mockSetOscillatorEnabled = vi.hoisted(() => vi.fn());
const mockUseHasProcessorInSignalChain = vi.hoisted(() => vi.fn());
const mockUseAllParametersFor = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  logger: {
    error: vi.fn(),
  },
  useMeterFrame: mockUseMeterFrame,
  useParameter: mockUseParameter,
  useHasProcessorInSignalChain: mockUseHasProcessorInSignalChain,
  useAllParametersFor: mockUseAllParametersFor,
}));

vi.mock('./ParameterSlider', () => ({
  ParameterSlider: ({ id }: { id: string }) => <div data-testid={`slider-${id}`} />,
}));

vi.mock('./ParameterSelect', () => ({
  ParameterSelect: ({ id }: { id: string }) => <div data-testid={`select-${id}`} />,
}));

describe('OscillatorProcessor', () => {
  beforeEach(() => {
    mockUseMeterFrame.mockReturnValue({ peak_l: 0.2, peak_r: 0.1 });
    mockSetOscillatorEnabled.mockReset();
    mockUseAllParametersFor.mockReturnValue({
      params: [
        { id: 'oscillator_enabled' },
        { id: 'oscillator_waveform' },
        { id: 'oscillator_frequency' },
        { id: 'oscillator_level' },
      ],
      isLoading: false,
      error: null,
      reload: vi.fn(),
    });
    mockUseParameter.mockReturnValue({
      param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: true },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    });
    mockUseHasProcessorInSignalChain.mockReturnValue(true);
  });

  it('reflects oscillator enabled value changes in visual state', () => {
    const { rerender } = render(<OscillatorProcessor />);

    expect(screen.getByText('Output signal (post-chain)')).toBeInTheDocument();
    expect(screen.getByText('Signal at output')).toBeInTheDocument();
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );

    mockUseParameter.mockReturnValue({
      param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: false },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    });

    rerender(<OscillatorProcessor />);

    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();
    expect(screen.getByText('Off')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
  });

  it('shows Off signal status when oscillator is disabled even if output meter has signal', () => {
    mockUseMeterFrame.mockReturnValue({ peak_l: 0.8, peak_r: 0.7 });
    mockUseParameter.mockReturnValue({
      param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: false },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    });

    render(<OscillatorProcessor />);

    expect(screen.getByText('Off')).toBeInTheDocument();
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();
    expect(screen.queryByText('Signal at output')).not.toBeInTheDocument();
  });

  it('can toggle oscillator off and back on', () => {
    let enabledValue = true;

    mockUseParameter.mockImplementation(() => ({
      param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: enabledValue },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    }));

    mockSetOscillatorEnabled.mockImplementation(async (nextValue: boolean) => {
      enabledValue = nextValue;
    });

    const { rerender } = render(<OscillatorProcessor />);
    const toggle = screen.getByRole('button', { name: 'Toggle oscillator output' });

    fireEvent.click(toggle);

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(false);

    rerender(<OscillatorProcessor />);
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Toggle oscillator output' }));

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(true);

    rerender(<OscillatorProcessor />);
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  it('renders oscillator frequency and level controls inside oscillator component', () => {
    render(<OscillatorProcessor />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
    expect(screen.getByTestId('select-oscillator_waveform')).toBeInTheDocument();
    expect(screen.getByTestId('slider-oscillator_frequency')).toBeInTheDocument();
    expect(screen.getByTestId('slider-oscillator_level')).toBeInTheDocument();
  });

  it('clears transient connection error UI after reconnect-driven hook recovery', () => {
    mockUseParameter
      .mockReturnValueOnce({
        param: null,
        setValue: mockSetOscillatorEnabled,
        isLoading: true,
        error: null,
      })
      .mockReturnValueOnce({
        param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: true },
        setValue: mockSetOscillatorEnabled,
        isLoading: false,
        error: null,
      });

    const { rerender } = render(<OscillatorProcessor />);

    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();

    rerender(<OscillatorProcessor />);

    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  it('resolves legacy param_N IDs when canonical oscillator IDs are unavailable', () => {
    mockUseAllParametersFor.mockReturnValue({
      params: [{ id: 'param_0' }, { id: 'param_1' }, { id: 'param_2' }, { id: 'param_3' }],
      isLoading: false,
      error: null,
      reload: vi.fn(),
    });

    mockUseParameter.mockImplementation((id: string) => ({
      param: { id, name: 'Oscillator Enabled', value: true },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    }));

    render(<OscillatorProcessor />);

    expect(mockUseParameter).toHaveBeenCalledWith('param_0');
    expect(screen.getByTestId('select-param_1')).toBeInTheDocument();
    expect(screen.getByTestId('slider-param_2')).toBeInTheDocument();
    expect(screen.getByTestId('slider-param_3')).toBeInTheDocument();
  });
});
