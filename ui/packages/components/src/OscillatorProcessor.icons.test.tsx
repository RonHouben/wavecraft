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

import { TestToneProcessor } from '../../../../sdk-template/ui/src/processors/TestToneProcessor';

type NumericOrBooleanParameter = ParameterInfo<number | boolean>;

function makeParameter(overrides: Partial<NumericOrBooleanParameter>): NumericOrBooleanParameter {
  return {
    id: 'test_tone_frequency',
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

describe('sdk-template TestToneProcessor semantics', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSetParameter.mockReset();
    mockSetParameter.mockResolvedValue(undefined);
  });

  it('renders test tone controls without waveform selection', () => {
    const parameterMap = new Map<string, NumericOrBooleanParameter>([
      [
        'test_tone_bypass',
        makeParameter({ id: 'test_tone_bypass', name: 'Bypass', type: 'bool', value: false }),
      ],
      [
        'test_tone_enabled',
        makeParameter({ id: 'test_tone_enabled', name: 'Enabled', type: 'bool', value: true }),
      ],
      [
        'test_tone_frequency',
        makeParameter({
          id: 'test_tone_frequency',
          name: 'Frequency',
          type: 'float',
          value: 440,
        }),
      ],
      [
        'test_tone_level',
        makeParameter({
          id: 'test_tone_level',
          name: 'Level',
          type: 'float',
          value: 0.5,
          min: 0,
          max: 1,
          unit: '%',
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

    render(<TestToneProcessor />);

    expect(screen.getByRole('heading', { name: 'Test Tone' })).toBeInTheDocument();
    expect(screen.queryByRole('radio')).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText('Frequency'), {
      target: { value: '880' },
    });

    expect(mockSetParameter).toHaveBeenCalledWith('test_tone_frequency', 880);
  });
});
