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
    return <div data-testid="tone-filter-processor-card">{props.children}</div>;
  },
}));

import { ToneFilterProcessor } from './ToneFilterProcessor';

const MODE_PARAMETER_ID = 'tone_filter_mode';
const CUTOFF_PARAMETER_ID = 'tone_filter_cutoff_hz';
const RESONANCE_PARAMETER_ID = 'tone_filter_resonance_q';

const modeParameter = {
  id: MODE_PARAMETER_ID,
  name: 'Mode',
  value: 0,
  default: 0,
  min: 0,
  max: 2,
  unit: '',
  variants: ['Low-pass', 'High-pass', 'Band-pass'],
  type: 'enum' as const,
};

const cutoffParameter = {
  id: CUTOFF_PARAMETER_ID,
  name: 'Cutoff',
  value: 1200,
  default: 1200,
  min: 20,
  max: 20_000,
  unit: 'Hz',
  type: 'float' as const,
};

const resonanceParameter = {
  id: RESONANCE_PARAMETER_ID,
  name: 'Resonance',
  value: 0.7,
  default: 0.7,
  min: 0.1,
  max: 10,
  unit: 'Q',
  type: 'float' as const,
};

describe('ToneFilterProcessor', () => {
  const setModeValue = vi.fn();
  const setCutoffValue = vi.fn();
  const setResonanceValue = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    mockUseParameter.mockImplementation((parameterId: string) => {
      if (parameterId === MODE_PARAMETER_ID) {
        return {
          param: modeParameter,
          setValue: setModeValue,
          isLoading: false,
          error: null,
        };
      }

      if (parameterId === CUTOFF_PARAMETER_ID) {
        return {
          param: cutoffParameter,
          setValue: setCutoffValue,
          isLoading: false,
          error: null,
        };
      }

      if (parameterId === RESONANCE_PARAMETER_ID) {
        return {
          param: resonanceParameter,
          setValue: setResonanceValue,
          isLoading: false,
          error: null,
        };
      }

      throw new Error(`Unexpected parameter id: ${parameterId}`);
    });
  });

  it('binds derived ids and forwards ProcessorCard props', () => {
    render(<ToneFilterProcessor hideWhenNotInSignalChain className="test-class" />);

    expect(mockUseParameter).toHaveBeenCalledWith(MODE_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(CUTOFF_PARAMETER_ID);
    expect(mockUseParameter).toHaveBeenCalledWith(RESONANCE_PARAMETER_ID);

    const cardProps = mockProcessorCardRender.mock.calls[0]?.[0] as MockProcessorCardProps;
    expect(cardProps.processorId).toBe('tone_filter');
    expect(cardProps.title).toBe('Tone Filter');
    expect(cardProps.subtitle).toBe('Filter');
    expect(cardProps.hideWhenNotInSignalChain).toBe(true);
    expect(cardProps.className).toContain('h-full');
    expect(cardProps.className).toContain('w-full');
    expect(cardProps.className).toContain('test-class');
  });

  it('renders controls from metadata and wires interactions to setters', () => {
    render(<ToneFilterProcessor />);

    const modeSelect = screen.getByRole('combobox', { name: modeParameter.name });
    expect(modeSelect).toHaveAttribute('id', `param-${modeParameter.id}`);
    expect(screen.getByRole('option', { name: 'Low-pass' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'High-pass' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Band-pass' })).toBeInTheDocument();

    fireEvent.change(modeSelect, { target: { value: '1' } });
    expect(setModeValue).toHaveBeenCalledWith(1);

    const cutoffKnob = screen.getByRole('slider', { name: cutoffParameter.name });
    const resonanceKnob = screen.getByRole('slider', { name: resonanceParameter.name });

    expect(cutoffKnob).toHaveAttribute('id', `param-${cutoffParameter.id}`);
    expect(resonanceKnob).toHaveAttribute('id', `param-${resonanceParameter.id}`);

    fireEvent.change(cutoffKnob, { target: { value: '3500' } });
    fireEvent.change(resonanceKnob, { target: { value: '1.2' } });

    expect(setCutoffValue).toHaveBeenCalledWith(3500);
    expect(setResonanceValue).toHaveBeenCalledWith(1.2);
  });
});
