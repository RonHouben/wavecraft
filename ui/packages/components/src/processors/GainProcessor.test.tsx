import { fireEvent, render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseParameter = vi.hoisted(() => vi.fn());
const mockProcessorCardRender = vi.hoisted(() => vi.fn());

interface MockProcessorCardProps {
  readonly signalChainProcessorId?: string;
  readonly bypassParameterId: string;
  readonly switchId?: string;
  readonly subtitle?: string;
  readonly title: string;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
  readonly onSwitchChange?: (checked: boolean) => void;
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

    return (
      <div data-testid="gain-processor-card" data-switch-id={props.switchId}>
        <button
          type="button"
          data-testid="gain-processor-switch"
          onClick={() => {
            props.onSwitchChange?.(true);
          }}
        >
          Toggle
        </button>
        {props.children}
      </div>
    );
  },
}));

import { GainProcessor } from './GainProcessor';

const SIGNAL_CHAIN_PROCESSOR_ID = 'input_trim';
const BYPASS_PARAMETER_ID = 'input_trim_bypass';
const ENABLED_PARAMETER_ID = 'test_tone_enabled';
const LEVEL_PARAMETER_ID = 'input_trim_level';

const enabledParameter = {
  id: 'gain_enabled_visible',
  name: 'Enabled',
  value: true,
  default: true,
  min: 0,
  max: 1,
  unit: '',
  type: 'bool' as const,
};

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
  const setEnabledValue = vi.fn();
  const setLevelValue = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    mockUseParameter.mockImplementation((parameterId: string) => {
      if (parameterId === ENABLED_PARAMETER_ID) {
        return {
          param: enabledParameter,
          setValue: setEnabledValue,
          isLoading: false,
          error: null,
        };
      }

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
        bypassParameterId={BYPASS_PARAMETER_ID}
        enabledParameterId={ENABLED_PARAMETER_ID}
        levelParameterId={LEVEL_PARAMETER_ID}
        title="Gain"
        subtitle="Level"
        hideWhenNotInSignalChain
        className="test-class"
      />
    );

    expect(mockUseParameter).toHaveBeenCalledWith(ENABLED_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(LEVEL_PARAMETER_ID);

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.signalChainProcessorId).toBe(SIGNAL_CHAIN_PROCESSOR_ID);
    expect(cardProps.bypassParameterId).toBe(BYPASS_PARAMETER_ID);
    expect(cardProps.title).toBe('Gain');
    expect(cardProps.subtitle).toBe('Level');
    expect(cardProps.hideWhenNotInSignalChain).toBe(true);
    expect(cardProps.switchId).toBe(`param-${enabledParameter.id}-switch`);
    expect(cardProps.className).toContain('h-full');
    expect(cardProps.className).toContain('w-full');
    expect(cardProps.className).toContain('test-class');
  });

  it('wires ProcessorCard switch callback to enabled parameter setter', () => {
    render(
      <GainProcessor
        processorId={SIGNAL_CHAIN_PROCESSOR_ID}
        bypassParameterId={BYPASS_PARAMETER_ID}
        enabledParameterId={ENABLED_PARAMETER_ID}
        levelParameterId={LEVEL_PARAMETER_ID}
        title="Gain"
        subtitle="Level"
      />
    );

    fireEvent.click(screen.getByTestId('gain-processor-switch'));

    expect(setEnabledValue).toHaveBeenCalledWith(true);
  });

  it('renders level knob from metadata and wires knob changes to level setter', () => {
    render(
      <GainProcessor
        processorId={SIGNAL_CHAIN_PROCESSOR_ID}
        bypassParameterId={BYPASS_PARAMETER_ID}
        enabledParameterId={ENABLED_PARAMETER_ID}
        levelParameterId={LEVEL_PARAMETER_ID}
        title="Gain"
        subtitle="Level"
      />
    );

    const levelKnob = screen.getByRole('slider', { name: levelParameter.name });

    expect(levelKnob).toHaveAttribute('id', `param-${levelParameter.id}`);
    expect(levelKnob).toHaveAttribute('min', String(levelParameter.min));
    expect(levelKnob).toHaveAttribute('max', String(levelParameter.max));
    expect(screen.getByText('1.00 x')).toBeInTheDocument();

    fireEvent.change(levelKnob, { target: { value: '1.5' } });

    expect(setLevelValue).toHaveBeenCalledWith(1.5);
  });
});
