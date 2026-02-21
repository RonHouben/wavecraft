import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { OscillatorProcessor } from './OscillatorProcessor';

const mockUseHasProcessorInSignalChain = vi.hoisted(() => vi.fn());
const mockUseProcessorBypass = vi.hoisted(() => vi.fn());
const mockUseParameter = vi.hoisted(() => vi.fn());
const mockSetOscillatorEnabled = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  logger: {
    error: vi.fn(),
  },
  useHasProcessorInSignalChain: mockUseHasProcessorInSignalChain,
  useProcessorBypass: mockUseProcessorBypass,
  useParameter: mockUseParameter,
}));

vi.mock('./Processor', () => ({
  Processor: ({ id }: { id: string }) => <div data-testid="processor" data-processor-id={id} />,
}));

describe('OscillatorProcessor', () => {
  beforeEach(() => {
    mockSetOscillatorEnabled.mockReset();
    mockSetOscillatorEnabled.mockResolvedValue(undefined);
    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseProcessorBypass.mockReturnValue({ bypassed: false });
    mockUseParameter.mockReturnValue({
      param: { id: 'oscillator_enabled', value: true },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: null,
    });
  });

  it('renders processor for oscillator id', () => {
    render(<OscillatorProcessor />);

    expect(screen.getByTestId('processor')).toHaveAttribute('data-processor-id', 'oscillator');
  });

  it('hides when processor is not in signal chain and hideWhenNotInSignalChain is set', () => {
    mockUseHasProcessorInSignalChain.mockReturnValue(false);

    render(<OscillatorProcessor hideWhenNotInSignalChain />);

    expect(screen.queryByTestId('processor')).not.toBeInTheDocument();
  });

  it('forces oscillator enabled off when bypass becomes active', async () => {
    mockUseProcessorBypass.mockReturnValue({ bypassed: true });

    render(<OscillatorProcessor />);

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(false);
  });

  it('restores oscillator enabled when bypass is removed after forced disable', () => {
    const bypassState = { bypassed: true };
    const enabledState = { value: true };

    mockUseProcessorBypass.mockImplementation(() => bypassState);
    mockUseParameter.mockImplementation(() => ({
      param: { id: 'oscillator_enabled', value: enabledState.value },
      setValue: mockSetOscillatorEnabled,
      isLoading: false,
      error: null,
    }));

    const { rerender } = render(<OscillatorProcessor />);

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(false);

    enabledState.value = false;
    bypassState.bypassed = false;

    rerender(<OscillatorProcessor />);

    expect(mockSetOscillatorEnabled).toHaveBeenCalledWith(true);
  });
});
