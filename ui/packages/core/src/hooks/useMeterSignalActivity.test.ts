import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getMeterSignalLevel, useMeterSignalActivity } from './useMeterSignalActivity';

const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const DEFAULT_HOLD_MS = 220;

vi.mock('./useMeterFrame', () => ({
  useMeterFrame: mockUseMeterFrame,
}));

describe('useMeterSignalActivity', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  it('defaults to immediate active/inactive transitions without smoothing', () => {
    let frame = {
      peak_l: 0.003,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 1,
    };

    mockUseMeterFrame.mockImplementation(() => frame);

    const { result, rerender } = renderHook(() => useMeterSignalActivity());

    expect(result.current).toBe(true);
    expect(mockUseMeterFrame).toHaveBeenCalledWith(50);

    frame = {
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 2,
    };

    rerender();

    expect(result.current).toBe(false);
  });

  it('supports smoothing hold when explicitly enabled', () => {
    vi.useFakeTimers();

    let frame = {
      peak_l: 0.01,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 1,
    };

    mockUseMeterFrame.mockImplementation(() => frame);

    const { result, rerender } = renderHook(() =>
      useMeterSignalActivity({ smoothing: { enabled: true } })
    );

    expect(result.current).toBe(true);

    frame = {
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 2,
    };

    rerender();
    expect(result.current).toBe(true);

    act(() => {
      vi.advanceTimersByTime(DEFAULT_HOLD_MS - 1);
    });
    expect(result.current).toBe(true);

    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(result.current).toBe(false);
  });
});

describe('getMeterSignalLevel', () => {
  it('returns max value across peak/rms channels', () => {
    expect(
      getMeterSignalLevel({
        peak_l: 0.1,
        peak_r: 0.2,
        rms_l: 0.3,
        rms_r: 0.25,
        timestamp: 1,
      })
    ).toBe(0.3);
  });

  it('returns 0 when frame is missing', () => {
    expect(getMeterSignalLevel(null)).toBe(0);
  });
});
