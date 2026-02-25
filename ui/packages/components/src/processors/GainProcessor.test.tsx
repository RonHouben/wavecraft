import { fireEvent, render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseParameter = vi.hoisted(() => vi.fn());
const mockProcessorCardRender = vi.hoisted(() => vi.fn());

interface MockProcessorCardProps {
  readonly processorId: string;
  readonly subtitle?: string;
  readonly title: string;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
  readonly children: ReactNode;
}

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');

  return {
    ...actual,
    useParameter: mockUseParameter,
  };
});

vi.mock('./ProcessorCard', () => ({
  ProcessorCard: (props: Readonly<MockProcessorCardProps>): ReactNode => {
    mockProcessorCardRender(props);

    return <div data-testid="gain-processor-card">{props.children}</div>;
  },
}));

import { GainProcessor } from './GainProcessor';

const SIGNAL_CHAIN_PROCESSOR_ID = 'input_trim';
const LEVEL_PARAMETER_ID = 'input_trim_level';

const levelParameter = {
  id: 'gain_level_visible',
  name: 'Level',
  value: 1,
  default: 1,
  min: 0,
  max: 2,
  unit: 'x',
  type: 'float' as const,
};

describe('GainProcessor', () => {
  const setLevelValue = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    mockUseParameter.mockImplementation((parameterId: string) => {
      if (parameterId === LEVEL_PARAMETER_ID) {
        return {
          param: levelParameter,
          setValue: setLevelValue,
          isLoading: false,
          error: null,
        };
      }

      throw new Error(`Unexpected parameter id: ${parameterId}`);
    });
  });

  it('binds hook ids and forwards prop-driven card bindings', () => {
    render(
      <GainProcessor
        processorId={SIGNAL_CHAIN_PROCESSOR_ID}
        title="Gain"
        subtitle="Level"
        hideWhenNotInSignalChain
        className="test-class"
      />
    );

    expect(mockUseParameter).toHaveBeenCalledWith(`${SIGNAL_CHAIN_PROCESSOR_ID}_level`);

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.processorId).toBe(SIGNAL_CHAIN_PROCESSOR_ID);
    expect(cardProps.title).toBe('Gain');
    expect(cardProps.subtitle).toBe('Level');
    expect(cardProps.hideWhenNotInSignalChain).toBe(true);
    expect(cardProps.className).toContain('h-full');
    expect(cardProps.className).toContain('w-full');
    expect(cardProps.className).toContain('test-class');
  });

  it('renders level knob from metadata and wires knob changes to level setter', () => {
    render(<GainProcessor processorId={SIGNAL_CHAIN_PROCESSOR_ID} title="Gain" subtitle="Level" />);

    const levelKnob = screen.getByRole('slider', { name: levelParameter.name });

    expect(levelKnob).toHaveAttribute('id', `param-${levelParameter.id}`);
    expect(levelKnob).toHaveAttribute('min', String(levelParameter.min));
    expect(levelKnob).toHaveAttribute('max', String(levelParameter.max));
    expect(screen.getByText('1.00 x')).toBeInTheDocument();

    fireEvent.change(levelKnob, { target: { value: '1.5' } });

    expect(setLevelValue).toHaveBeenCalledWith(1.5);
  });
});
