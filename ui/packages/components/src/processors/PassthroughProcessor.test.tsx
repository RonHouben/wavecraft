import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockProcessorCardRender = vi.hoisted(() => vi.fn());

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

  it('renders a static passthrough glyph with truthful stage copy', () => {
    render(<PassthroughProcessor processorId={PROCESSOR_ID} />);

    expect(screen.getByTestId('passthrough-glyph')).toBeInTheDocument();
    expect(screen.getByTestId('passthrough-glyph-ring')).toHaveClass('border-plugin-border');
    expect(screen.getByTestId('passthrough-glyph-line')).toHaveClass('bg-accent');
    expect(screen.getByTestId('passthrough-glyph-input')).toHaveClass('border-accent-light');
    expect(screen.getByTestId('passthrough-glyph-output')).toHaveClass('bg-accent');
    expect(screen.getByText('Passes audio unchanged')).toBeInTheDocument();
    expect(screen.getByTestId('passthrough-stage-note')).toHaveTextContent(
      'Signal activity is shown at the plugin output, not at this stage.'
    );
  });
});
