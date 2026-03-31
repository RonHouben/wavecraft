import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockProcessorCardRender = vi.hoisted(() => vi.fn());
const mockUsePassthroughMeterSignalActivity = vi.hoisted(() =>
  vi.fn(() => ({
    isSignalActive: false,
    signalIntensity: 0,
    signalLevel: 0,
  }))
);
const mockGetMeterClipWarningIntensity = vi.hoisted(() => vi.fn(() => 0));

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
    getMeterClipWarningIntensity: mockGetMeterClipWarningIntensity,
    usePassthroughMeterSignalActivity: mockUsePassthroughMeterSignalActivity,
  };
});

import { PassthroughProcessor } from './PassthroughProcessor';

const PROCESSOR_ID = 'input_trim';

describe('PassthroughProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
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

  it('renders a Passthrough eye driven by the local signal meter state', () => {
    render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(screen.getByTestId('passthrough-eye')).toHaveAttribute('data-signal-active', 'false');
    expect(screen.getByTestId('passthrough-eye')).toHaveAttribute(
      'data-clip-warning-active',
      'false'
    );
    expect(screen.getByTestId('passthrough-eye-outer-glow')).toHaveClass('bg-plugin-border-strong');
    expect(screen.getByTestId('passthrough-eye-inner-glow')).toHaveClass('bg-accent');
    expect(screen.getByTestId('passthrough-eye-pupil')).toBeInTheDocument();
    expect(mockUsePassthroughMeterSignalActivity).toHaveBeenCalledOnce();
  });
});
