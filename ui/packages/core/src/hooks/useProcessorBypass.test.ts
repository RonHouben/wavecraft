import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useProcessorBypass } from './useProcessorBypass';

const setValueMock = vi.fn<(value: number | boolean) => Promise<void>>();

vi.mock('./useParameter', () => ({
  useParameter: vi.fn(() => ({
    param: {
      id: 'input_trim_bypass',
      name: 'Input Trim Bypass',
      type: 'bool',
      value: false,
      default: false,
      min: 0,
      max: 1,
    },
    setValue: setValueMock,
    isLoading: false,
    error: null,
  })),
}));

describe('useProcessorBypass', () => {
  beforeEach(() => {
    setValueMock.mockReset();
    setValueMock.mockResolvedValue();
  });

  it('maps processor id to bypass parameter id', () => {
    const { result } = renderHook(() => useProcessorBypass('input_trim'));

    expect(result.current.bypassParameterId).toBe('input_trim_bypass');
    expect(result.current.bypassed).toBe(false);
  });

  it('setBypassed writes boolean value through useParameter', async () => {
    const { result } = renderHook(() => useProcessorBypass('input_trim'));

    await act(async () => {
      await result.current.setBypassed(true);
    });

    expect(setValueMock).toHaveBeenCalledWith(true);
  });

  it('toggle flips current bypass state', async () => {
    const { result } = renderHook(() => useProcessorBypass('input_trim'));

    await act(async () => {
      await result.current.toggle();
    });

    expect(setValueMock).toHaveBeenCalledWith(true);
  });
});
