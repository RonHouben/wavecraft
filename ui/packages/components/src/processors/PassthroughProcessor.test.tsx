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
    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: false,
      signalIntensity: 0,
      signalLevel: 0,
    });
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
      intensityRange: {
        ceilingDb: -6,
        floorDb: -54,
      },
      smoothing: {
        enabled: true,
      },
    });
  });

  it('glows while signal is active and fades to idle when hook activity toggles', () => {
    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 1,
      signalLevel: 0.82,
    });

    const { rerender } = render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    const eye = screen.getByTestId('passthrough-eye');
    expect(eye).toHaveAttribute('data-signal-active', 'true');
    expect(eye).toHaveAttribute('data-signal-intensity', '1.000');
    expect(eye).toHaveAttribute('data-clip-warning-active', 'false');
    expect(eye).toHaveAttribute('data-clip-warning-intensity', '0.000');

    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 0.9,
      signalLevel: 0.97,
    });

    rerender(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(eye).toHaveAttribute('data-clip-warning-active', 'true');
    const activeClipWarningIntensity = Number.parseFloat(
      eye.getAttribute('data-clip-warning-intensity') ?? '0'
    );
    expect(activeClipWarningIntensity).toBeGreaterThan(0);

    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 0.65,
      signalLevel: 0.86,
    });

    rerender(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(eye).toHaveAttribute('data-clip-warning-active', 'true');

    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: false,
      signalIntensity: 0.15,
      signalLevel: 0.7,
    });

    rerender(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(eye).toHaveAttribute('data-signal-active', 'false');
    expect(eye).toHaveAttribute('data-signal-intensity', '0.150');
    expect(eye).toHaveAttribute('data-clip-warning-active', 'false');
    expect(eye).toHaveAttribute('data-clip-warning-intensity', '0.000');
  });

  it('renders bounded intensity-based inline styles for glow and pupil opacity', () => {
    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 0.5,
      signalLevel: 0.12,
    });

    render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    const outerGlow = screen.getByTestId('passthrough-eye-outer-glow');
    const clipAura = screen.getByTestId('passthrough-eye-clip-aura');
    const clipRing = screen.getByTestId('passthrough-eye-clip-ring');
    const innerGlow = screen.getByTestId('passthrough-eye-inner-glow');
    const pupil = screen.getByTestId('passthrough-eye-pupil');

    expect(outerGlow).toHaveStyle({ opacity: '0.44' });
    expect(clipAura).toHaveStyle({ opacity: '0', transform: 'scale(1)' });
    expect(clipRing).toHaveStyle({ opacity: '0' });
    expect(innerGlow).toHaveStyle({ opacity: '0.625', transform: 'scale(0.975)' });
    expect(pupil).toHaveStyle({ opacity: '0.75' });
  });

  it('switches eye clip-warning layers to warning color tokens only while clip warning is active', () => {
    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 0.8,
      signalLevel: 0.85,
    });

    const { rerender } = render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    const outerGlow = screen.getByTestId('passthrough-eye-outer-glow');
    const clipAura = screen.getByTestId('passthrough-eye-clip-aura');
    const clipRing = screen.getByTestId('passthrough-eye-clip-ring');
    const innerGlow = screen.getByTestId('passthrough-eye-inner-glow');

    expect(outerGlow).toHaveClass('bg-plugin-border-strong');
    expect(clipAura).toHaveClass('bg-accent');
    expect(clipRing).toHaveClass('border-accent-light');
    expect(innerGlow).toHaveClass('bg-accent');

    mockUseMeterSignalActivity.mockReturnValue({
      isSignalActive: true,
      signalIntensity: 0.95,
      signalLevel: 0.99,
    });

    rerender(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(outerGlow).toHaveClass('bg-meter-clip-dark');
    expect(outerGlow).not.toHaveClass('bg-plugin-border-strong');
    expect(clipAura).toHaveClass('bg-meter-clip');
    expect(clipAura).not.toHaveClass('bg-accent');
    expect(clipRing).toHaveClass('border-meter-clip');
    expect(clipRing).not.toHaveClass('border-accent-light');
    expect(innerGlow).toHaveClass('bg-meter-clip');
    expect(innerGlow).not.toHaveClass('bg-accent');
  });
});
