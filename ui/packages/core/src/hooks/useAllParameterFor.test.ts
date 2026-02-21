import { renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useAllParametersFor } from './useAllParameterFor';

const useAllParametersMock = vi.hoisted(() => vi.fn());
const reloadMock = vi.hoisted(() => vi.fn(async () => {}));

vi.mock('./useAllParameters', () => ({
  useAllParameters: useAllParametersMock,
}));

describe('useAllParametersFor', () => {
  beforeEach(() => {
    useAllParametersMock.mockReset();
    reloadMock.mockClear();

    useAllParametersMock.mockReturnValue({
      params: [
        {
          id: 'input_trim_bypass',
          name: 'Input Trim Bypass',
          type: 'bool',
          value: false,
          default: false,
          min: 0,
          max: 1,
        },
        {
          id: 'input_trim_level',
          name: 'Level',
          type: 'float',
          value: 1,
          default: 1,
          min: 0,
          max: 2,
        },
        {
          id: 'tone_filter_cutoff_hz',
          name: 'Cutoff',
          type: 'float',
          value: 1000,
          default: 1000,
          min: 20,
          max: 20000,
        },
      ],
      isLoading: false,
      error: null,
      reload: reloadMock,
    });
  });

  it('returns all parameters belonging to the requested processor (including bypass)', () => {
    const { result } = renderHook(() => useAllParametersFor('input_trim'));

    expect(result.current.params.map((param) => param.id)).toEqual([
      'input_trim_bypass',
      'input_trim_level',
    ]);
  });

  it('passes through loading/error/reload state from useAllParameters', () => {
    const { result } = renderHook(() => useAllParametersFor('input_trim'));

    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.reload).toBe(reloadMock);
  });

  it('supports the pluralized alias export', () => {
    const { result } = renderHook(() => useAllParametersFor('input_trim'));

    expect(result.current.params).toHaveLength(2);
    expect(result.current.processorId).toBe('input_trim');
  });
});
