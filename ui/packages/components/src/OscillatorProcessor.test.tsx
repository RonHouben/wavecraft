import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { OscillatorProcessor } from './OscillatorProcessor';

const mockProcessor = vi.hoisted(() =>
  vi.fn(({ id }: { id: string }) => <div data-testid="processor" data-processor-id={id} />)
);

vi.mock('./Processor', () => ({
  Processor: mockProcessor,
}));

describe('OscillatorProcessor', () => {
  beforeEach(() => {
    mockProcessor.mockClear();
  });

  it('renders processor for oscillator id', () => {
    render(<OscillatorProcessor />);

    expect(screen.getByTestId('processor')).toHaveAttribute('data-processor-id', 'oscillator');
  });

  it('hides when processor is not in signal chain and hideWhenNotInSignalChain is set', () => {
    render(<OscillatorProcessor hideWhenNotInSignalChain />);

    const lastCallIndex = mockProcessor.mock.calls.length - 1;
    const props = mockProcessor.mock.calls[lastCallIndex]?.[0];
    expect(props).toMatchObject({ id: 'oscillator', hideWhenNotInSignalChain: true });
  });

  it('defaults hideWhenNotInSignalChain to undefined when omitted', () => {
    render(<OscillatorProcessor />);

    const lastCallIndex = mockProcessor.mock.calls.length - 1;
    const props = mockProcessor.mock.calls[lastCallIndex]?.[0];
    expect(props).toMatchObject({ id: 'oscillator', hideWhenNotInSignalChain: undefined });
  });

  it('keeps oscillator processor id across rerenders', () => {
    const { rerender } = render(<OscillatorProcessor hideWhenNotInSignalChain />);

    rerender(<OscillatorProcessor />);

    expect(mockProcessor).toHaveBeenCalledTimes(2);
    expect(mockProcessor.mock.calls[0]?.[0]).toMatchObject({ id: 'oscillator' });
    expect(mockProcessor.mock.calls[1]?.[0]).toMatchObject({ id: 'oscillator' });
  });
});
