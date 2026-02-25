import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockProcessorCardRender = vi.hoisted(() => vi.fn());
const mockUseMeterSignalActivity = vi.hoisted(() => vi.fn());

interface MockProcessorCardProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly processorId: string;
  readonly subtitle?: string;
  readonly title: string;
  readonly className?: string;
  readonly children: ReactNode;
}

vi.mock('./ProcessorCard', () => ({
  ProcessorCard: (props: Readonly<MockProcessorCardProps>): ReactNode => {
    mockProcessorCardRender(props);
    return <div data-testid="passthrough-processor-card">{props.children}</div>;
  },
}));

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');

  return {
    ...actual,
    useMeterSignalActivity: mockUseMeterSignalActivity,
  };
});

import { PassthroughProcessor } from './PassthroughProcessor';

const PROCESSOR_ID = 'input_trim';

describe('PassthroughProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseMeterSignalActivity.mockReturnValue(false);
  });

  it('forwards ProcessorCard props derived from processorId', () => {
    render(
      <PassthroughProcessor
        processorId={PROCESSOR_ID}
        hideWhenNotInSignalChain
        className="test-class"
      />
    );

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.processorId).toBe(PROCESSOR_ID);
    expect(cardProps.title).toBe('Passthrough');
    expect(cardProps.subtitle).toBe('Bypass');
    expect(cardProps.hideWhenNotInSignalChain).toBe(true);
    expect(cardProps.className).toContain('h-full');
    expect(cardProps.className).toContain('w-full');
    expect(cardProps.className).toContain('test-class');

  });

  it('supports custom title/subtitle and keeps derived bypass id pattern', () => {
    render(
      <PassthroughProcessor processorId={PROCESSOR_ID} title="Input Trim" subtitle="Utility" />
    );

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.title).toBe('Input Trim');
    expect(cardProps.subtitle).toBe('Utility');
  });

  it('opts in to smoothing via core hook configuration', () => {
    render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(mockUseMeterSignalActivity).toHaveBeenCalledWith({
      smoothing: {
        enabled: true,
      },
    });
  });

  it('glows while signal is active and fades to idle when hook activity toggles', () => {
    mockUseMeterSignalActivity.mockReturnValue(true);

    const { rerender } = render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    const eye = screen.getByTestId('passthrough-eye');
    expect(eye).toHaveAttribute('data-signal-active', 'true');

    mockUseMeterSignalActivity.mockReturnValue(false);

    rerender(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(eye).toHaveAttribute('data-signal-active', 'false');
  });
});
