import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { InputTrimProcessor, OscillatorProcessor } from './compat';
import type { ProcessorParameter } from './Processor';

const mockProcessor = vi.hoisted(() =>
  vi.fn(({ id }: { id: string }) => <div data-testid="processor" data-processor-id={id} />)
);

vi.mock('./Processor', () => ({
  Processor: mockProcessor,
}));

describe('OscillatorProcessor', () => {
  const parameters: ProcessorParameter[] = [];

  beforeEach(() => {
    mockProcessor.mockClear();
  });

  it('renders processor for oscillator id', () => {
    render(<OscillatorProcessor parameters={parameters} />);

    expect(screen.getByTestId('processor')).toHaveAttribute('data-processor-id', 'oscillator');
  });

  it('forwards title and parameters props to Processor', () => {
    render(<OscillatorProcessor title="Oscillator" parameters={parameters} />);

    const lastCallIndex = mockProcessor.mock.calls.length - 1;
    const props = mockProcessor.mock.calls[lastCallIndex]?.[0];
    expect(props).toMatchObject({ id: 'oscillator', title: 'Oscillator', parameters });
  });

  it('omits title when not provided', () => {
    render(<OscillatorProcessor parameters={parameters} />);

    const lastCallIndex = mockProcessor.mock.calls.length - 1;
    const props = mockProcessor.mock.calls[lastCallIndex]?.[0] as
      | { id: string; title?: string; parameters: ProcessorParameter[] }
      | undefined;
    expect(props).toBeDefined();
    expect(props).toMatchObject({ id: 'oscillator', parameters });
    expect(props?.title).toBeUndefined();
  });

  it('keeps oscillator processor id across rerenders', () => {
    const { rerender } = render(<OscillatorProcessor title="Oscillator" parameters={parameters} />);

    rerender(<OscillatorProcessor parameters={parameters} />);

    expect(mockProcessor).toHaveBeenCalledTimes(2);
    expect(mockProcessor.mock.calls[0]?.[0]).toMatchObject({ id: 'oscillator' });
    expect(mockProcessor.mock.calls[1]?.[0]).toMatchObject({ id: 'oscillator' });
  });

  it('keeps input trim compat wrapper wired to processor id', () => {
    render(<InputTrimProcessor parameters={parameters} />);

    const lastCallIndex = mockProcessor.mock.calls.length - 1;
    const props = mockProcessor.mock.calls[lastCallIndex]?.[0] as
      | { id: string; parameters: ProcessorParameter[] }
      | undefined;
    expect(props).toBeDefined();
    expect(props).toMatchObject({ id: 'input_trim', parameters });
    expect(screen.getByTestId('processor')).toHaveAttribute('data-processor-id', 'input_trim');
  });
});
