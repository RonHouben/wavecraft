import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { OscillatorControl } from './OscillatorControl';

const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const mockUseParameter = vi.hoisted(() => vi.fn());
const mockSetOscillatorEnabled = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  logger: {
    error: vi.fn(),
  },
  useMeterFrame: mockUseMeterFrame,
  useParameter: mockUseParameter,
}));

vi.mock('./ParameterSlider', () => ({
  ParameterSlider: ({ id }: { id: string }) => <div data-testid={`slider-${id}`} />,
}));

describe('OscillatorControl', () => {
  beforeEach(() => {
    mockUseMeterFrame.mockReturnValue({ peak_l: 0.2, peak_r: 0.1 });
    mockSetOscillatorEnabled.mockReset();
    mockUseParameter.mockReturnValue({
      param: { id: 'oscillator_enabled', name: 'Oscillator Enabled', value: true },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: undefined,
    });
  });

  it('reflects oscillator enabled value changes in visual state', () => {
    const { rerender } = render(<OscillatorControl />);

    expect(screen.getByText('Oscillator signal')).toBeInTheDocument();
    expect(screen.getByText('Producing')).toBeInTheDocument();
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

    rerender(<OscillatorControl />);

    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
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

    const { rerender } = render(<OscillatorControl />);
    const toggle = screen.getByRole('button', { name: 'Toggle oscillator output' });

    fireEvent.click(toggle);

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(false);

    rerender(<OscillatorControl />);
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Toggle oscillator output' }));

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(true);

    rerender(<OscillatorControl />);
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  it('renders oscillator frequency and level controls inside oscillator component', () => {
    render(<OscillatorControl />);

    expect(screen.getByTestId('oscillator-control')).toBeInTheDocument();
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

    const { rerender } = render(<OscillatorControl />);

    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();

    rerender(<OscillatorControl />);

    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });
});
