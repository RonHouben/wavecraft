import { fireEvent, render, screen } from '@testing-library/react';
import type { ParameterInfo } from '@wavecraft/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseHasProcessorInSignalChain = vi.hoisted(() => vi.fn());
const mockUseParameter = vi.hoisted(() => vi.fn());
const mockSetParameter = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');
  return {
    ...actual,
    useHasProcessorInSignalChain: mockUseHasProcessorInSignalChain,
    useParameter: mockUseParameter,
  };
});

import { OscillatorProcessor } from '../../../../sdk-template/ui/src/processors/OscillatorProcessor';

type NumericOrBooleanParameter = ParameterInfo<number | boolean>;

function makeParameter(overrides: Partial<NumericOrBooleanParameter>): NumericOrBooleanParameter {
  return {
    id: 'oscillator_frequency',
    name: 'Frequency',
    type: 'float',
    value: 440,
    default: 440,
    min: 20,
    max: 20_000,
    unit: 'Hz',
    ...overrides,
  };
}

describe('sdk-template OscillatorProcessor waveform icons', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSetParameter.mockReset();
    mockSetParameter.mockResolvedValue(undefined);
  });

  it('renders waveform buttons with icons and forwards selection', () => {
    const parameterMap = new Map<string, NumericOrBooleanParameter>([
      [
        'oscillator_bypass',
        makeParameter({ id: 'oscillator_bypass', name: 'Bypass', type: 'bool', value: false }),
      ],
      [
        'oscillator_enabled',
        makeParameter({ id: 'oscillator_enabled', name: 'Enabled', type: 'bool', value: true }),
      ],
      [
        'oscillator_waveform',
        makeParameter({
          id: 'oscillator_waveform',
          name: 'Waveform',
          type: 'enum',
          value: 0,
          variants: ['Sine', 'Square', 'Saw', 'Triangle'],
        }),
      ],
      [
        'oscillator_frequency',
        makeParameter({ id: 'oscillator_frequency', name: 'Frequency', type: 'float', value: 440 }),
      ],
      [
        'oscillator_level',
        makeParameter({
          id: 'oscillator_level',
          name: 'Level',
          type: 'float',
          value: -6,
          min: -24,
          max: 0,
          unit: 'dB',
        }),
      ],
    ]);

    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseParameter.mockImplementation(<T extends number | boolean>(id: string) => {
      const param = parameterMap.get(id) as ParameterInfo<T> | undefined;
      return {
        param,
        setValue: (value: T): void => {
          void mockSetParameter(id, value);
        },
      };
    });

    render(<OscillatorProcessor />);

    expect(
      document.querySelector('[role="radio"][value="Sine"] [data-waveform-icon="sine"]')
    ).toBeInTheDocument();
    expect(
      document.querySelector('[role="radio"][value="Square"] [data-waveform-icon="square"]')
    ).toBeInTheDocument();
    expect(
      document.querySelector('[role="radio"][value="Saw"] [data-waveform-icon="saw"]')
    ).toBeInTheDocument();
    expect(
      document.querySelector('[role="radio"][value="Triangle"] [data-waveform-icon="triangle"]')
    ).toBeInTheDocument();

    const squareWaveformOption = document.querySelector<HTMLButtonElement>(
      '[role="radio"][value="Square"]'
    );
    expect(squareWaveformOption).toBeInTheDocument();

    fireEvent.click(squareWaveformOption as HTMLButtonElement);

    expect(mockSetParameter).toHaveBeenCalledWith('oscillator_waveform', 1);
  });
});
