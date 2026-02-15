import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ParameterSlider } from './ParameterSlider';

const mockUseParameter = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  logger: { error: vi.fn() },
  useParameter: mockUseParameter,
}));

describe('ParameterSlider reconnect recovery', () => {
  it('recovers from transient connection race once hook reconnects and reloads', () => {
    mockUseParameter
      .mockReturnValueOnce({
        param: null,
        setValue: vi.fn(),
        isLoading: true,
        error: null,
      })
      .mockReturnValueOnce({
        param: {
          id: 'oscillator_frequency',
          name: 'Oscillator Frequency',
          type: 'float',
          value: 0.44,
          default: 0.44,
          min: 20,
          max: 5000,
          unit: 'Hz',
        },
        setValue: vi.fn(),
        isLoading: false,
        error: null,
      });

    const { rerender } = render(<ParameterSlider id="oscillator_frequency" />);

    expect(screen.getByText('Loading oscillator_frequency...')).toBeInTheDocument();
    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();

    rerender(<ParameterSlider id="oscillator_frequency" />);

    expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
    expect(screen.getByText('Oscillator Frequency')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toBeInTheDocument();
  });
});
