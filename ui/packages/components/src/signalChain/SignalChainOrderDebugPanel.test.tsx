import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

const mockUseSignalChainOrder = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useSignalChainOrder: mockUseSignalChainOrder,
}));

import { SignalChainOrderDebugPanel } from './SignalChainOrderDebugPanel';

describe('SignalChainOrderDebugPanel', () => {
  it('renders backend-reported slots in order', () => {
    mockUseSignalChainOrder.mockReturnValue({
      order: [
        { id: 'input_trim', type: 'processor' },
        { id: 'oscilloscope_tap', type: 'tap' },
      ],
      setOrder: vi.fn(),
      isLoading: false,
      error: null,
    });

    render(<SignalChainOrderDebugPanel />);

    expect(screen.getByTestId('signal-chain-order-debug-panel')).toBeInTheDocument();
    expect(screen.getByText('Backend signal chain')).toBeInTheDocument();
    expect(screen.getByText('input_trim')).toBeInTheDocument();
    expect(screen.getByText('oscilloscope_tap')).toBeInTheDocument();
    expect(screen.getByText('processor')).toBeInTheDocument();
    expect(screen.getByText('tap')).toBeInTheDocument();
    expect(screen.getByText('2 slots')).toBeInTheDocument();
  });

  it('renders loading state while waiting for backend order', () => {
    mockUseSignalChainOrder.mockReturnValue({
      order: [],
      setOrder: vi.fn(),
      isLoading: true,
      error: null,
    });

    render(<SignalChainOrderDebugPanel />);

    expect(screen.getByTestId('signal-chain-order-debug-loading')).toBeInTheDocument();
    expect(screen.queryByTestId('signal-chain-order-debug-list')).not.toBeInTheDocument();
  });

  it('renders an error message when the backend order cannot be loaded', () => {
    mockUseSignalChainOrder.mockReturnValue({
      order: [],
      setOrder: vi.fn(),
      isLoading: false,
      error: { code: -32000, message: 'Transport unavailable' },
    });

    render(<SignalChainOrderDebugPanel />);

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Unable to read backend signal chain: Transport unavailable'
    );
  });
});
