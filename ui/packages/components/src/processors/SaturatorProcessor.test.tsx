import { fireEvent, render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseParameter = vi.hoisted(() => vi.fn());
const mockProcessorCardRender = vi.hoisted(() => vi.fn());

interface MockProcessorCardProps {
  readonly hideWhenNotInSignalChain?: boolean;
  readonly processorId: string;
  readonly subtitle?: string;
  readonly title: string;
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
    return <div data-testid="saturator-processor-card">{props.children}</div>;
  },
}));

import { SaturatorProcessor } from './SaturatorProcessor';

const DRIVE_PARAMETER_ID = 'soft_clip_drive_db';
const OUTPUT_PARAMETER_ID = 'soft_clip_output_db';
const MIX_PARAMETER_ID = 'soft_clip_mix';
const TONE_PARAMETER_ID = 'soft_clip_tone';

describe('SaturatorProcessor', () => {
  const setDriveValue = vi.fn();
  const setOutputValue = vi.fn();
  const setMixValue = vi.fn();
  const setToneValue = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    mockUseParameter.mockImplementation((parameterId: string) => {
      if (parameterId === DRIVE_PARAMETER_ID) {
        return {
          param: {
            id: DRIVE_PARAMETER_ID,
            name: 'Drive',
            value: 12,
            default: 12,
            min: 0,
            max: 30,
            unit: 'dB',
            type: 'float' as const,
          },
          setValue: setDriveValue,
          isLoading: false,
          error: null,
        };
      }

      if (parameterId === OUTPUT_PARAMETER_ID) {
        return {
          param: {
            id: OUTPUT_PARAMETER_ID,
            name: 'Output',
            value: 0,
            default: 0,
            min: -24,
            max: 24,
            unit: 'dB',
            type: 'float' as const,
          },
          setValue: setOutputValue,
          isLoading: false,
          error: null,
        };
      }

      if (parameterId === MIX_PARAMETER_ID) {
        return {
          param: {
            id: MIX_PARAMETER_ID,
            name: 'Mix',
            value: 1,
            default: 1,
            min: 0,
            max: 1,
            unit: '%',
            type: 'float' as const,
          },
          setValue: setMixValue,
          isLoading: false,
          error: null,
        };
      }

      if (parameterId === TONE_PARAMETER_ID) {
        return {
          param: {
            id: TONE_PARAMETER_ID,
            name: 'Tone',
            value: 0.55,
            default: 0.55,
            min: 0,
            max: 1,
            unit: '%',
            type: 'float' as const,
          },
          setValue: setToneValue,
          isLoading: false,
          error: null,
        };
      }

      throw new Error(`Unexpected parameter id: ${parameterId}`);
    });
  });

  it('binds derived IDs and forwards ProcessorCard props', () => {
    render(<SaturatorProcessor hideWhenNotInSignalChain className="test-class" />);

    expect(mockUseParameter).toHaveBeenCalledWith(DRIVE_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(OUTPUT_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(MIX_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(TONE_PARAMETER_ID);

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.processorId).toBe('soft_clip');
    expect(cardProps.title).toBe('Saturator');
    expect(cardProps.subtitle).toBe('Warm Soft Clip');
    expect(cardProps.hideWhenNotInSignalChain).toBe(true);
    expect(cardProps.className).toContain('h-full');
    expect(cardProps.className).toContain('w-full');
    expect(cardProps.className).toContain('test-class');
  });

  it('renders all control knobs and wires interactions to setters', () => {
    render(<SaturatorProcessor />);

    const driveKnob = screen.getByRole('slider', { name: 'Drive' });
    const outputKnob = screen.getByRole('slider', { name: 'Output' });
    const mixKnob = screen.getByRole('slider', { name: 'Mix' });
    const toneKnob = screen.getByRole('slider', { name: 'Tone' });

    expect(driveKnob).toHaveAttribute('id', `param-${DRIVE_PARAMETER_ID}`);
    expect(outputKnob).toHaveAttribute('id', `param-${OUTPUT_PARAMETER_ID}`);
    expect(mixKnob).toHaveAttribute('id', `param-${MIX_PARAMETER_ID}`);
    expect(toneKnob).toHaveAttribute('id', `param-${TONE_PARAMETER_ID}`);

    expect(outputKnob).toHaveAttribute('min', '-24');
    expect(outputKnob).toHaveAttribute('max', '24');
    expect(outputKnob).toHaveAttribute('value', '0');
    expect(mixKnob).toHaveAttribute('min', '0');
    expect(mixKnob).toHaveAttribute('max', '1');
    expect(mixKnob).toHaveAttribute('value', '1');
    expect(toneKnob).toHaveAttribute('min', '0');
    expect(toneKnob).toHaveAttribute('max', '1');
    expect(toneKnob).toHaveAttribute('value', '0.55');

    fireEvent.change(driveKnob, { target: { value: '18' } });
    fireEvent.change(outputKnob, { target: { value: '-6' } });
    fireEvent.change(mixKnob, { target: { value: '0.4' } });
    fireEvent.change(toneKnob, { target: { value: '0.8' } });

    expect(setDriveValue).toHaveBeenCalledWith(18);
    expect(setOutputValue).toHaveBeenCalledWith(-6);
    expect(setMixValue).toHaveBeenCalledWith(0.4);
    expect(setToneValue).toHaveBeenCalledWith(0.8);
  });
});
