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
const BYPASS_PARAMETER_ID = `${PROCESSOR_ID}_bypass`;

describe('PassthroughProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('derives bypass id from processorId and forwards ProcessorCard props', () => {
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

    expect(screen.getByTestId('passthrough-bypass-parameter-id')).toHaveTextContent(
      BYPASS_PARAMETER_ID
    );
  });

  it('supports custom title/subtitle and keeps derived bypass id pattern', () => {
    render(
      <PassthroughProcessor
        processorId={PROCESSOR_ID}
        title="Input Trim"
        subtitle="Utility"
      />
    );

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.title).toBe('Input Trim');
    expect(cardProps.subtitle).toBe('Utility');
  });

});
