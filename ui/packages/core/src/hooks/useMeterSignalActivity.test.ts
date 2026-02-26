import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  getMeterClipWarningIntensity,
  getMeterSignalIntensity,
  getMeterSignalLevel,
  useMeterSignalActivity,
} from './useMeterSignalActivity';
import { dbToLinear } from '../utils/audio-math';

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

    expect(result.current.isSignalActive).toBe(true);
    expect(mockUseMeterFrame).toHaveBeenCalledWith(50);

    frame = {
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 2,
    };

    rerender();

    expect(result.current.isSignalActive).toBe(false);
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

    expect(result.current.isSignalActive).toBe(true);

    frame = {
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 2,
    };

    rerender();
    expect(result.current.isSignalActive).toBe(true);

    act(() => {
      vi.advanceTimersByTime(DEFAULT_HOLD_MS - 1);
    });
    expect(result.current.isSignalActive).toBe(true);

    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(result.current.isSignalActive).toBe(false);
  });

  it('computes signalIntensity using bounded dB mapping defaults', () => {
    const frame = {
      peak_l: 0.1,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 1,
    };

    mockUseMeterFrame.mockReturnValue(frame);

    const { result } = renderHook(() => useMeterSignalActivity());

    expect(result.current.signalLevel).toBe(0.1);
    expect(result.current.signalIntensity).toBeCloseTo(0.583, 3);
  });

  it('supports custom dB floor and ceiling clamp bounds for signalIntensity', () => {
    const frame = {
      peak_l: 0.01,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 1,
    };

    mockUseMeterFrame.mockReturnValue(frame);

    const { result } = renderHook(() =>
      useMeterSignalActivity({ intensityRange: { floorDb: -40, ceilingDb: -6 } })
    );

    expect(result.current.signalIntensity).toBeCloseTo(0.0, 3);
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

describe('getMeterSignalIntensity', () => {
  it('returns 0 for silence and invalid values', () => {
    expect(getMeterSignalIntensity(0)).toBe(0);
    expect(getMeterSignalIntensity(-1)).toBe(0);
    expect(getMeterSignalIntensity(Number.NaN)).toBe(0);
  });

  it('maps level to normalized intensity using default dB bounds', () => {
    expect(getMeterSignalIntensity(1)).toBeCloseTo(1, 5);
    expect(getMeterSignalIntensity(0.1)).toBeCloseTo(0.583, 3);
  });

  it('clamps below floor and above ceiling dB', () => {
    expect(getMeterSignalIntensity(0.001, { floorDb: -40, ceilingDb: -6 })).toBe(0);
    expect(getMeterSignalIntensity(1, { floorDb: -40, ceilingDb: -6 })).toBe(1);
  });

  it('handles reversed dB range input by normalizing bounds', () => {
    expect(getMeterSignalIntensity(0.1, { floorDb: -6, ceilingDb: -40 })).toBeCloseTo(0.588, 3);
  });
});

describe('getMeterClipWarningIntensity', () => {
  it('returns 0 for silence and invalid values', () => {
    expect(getMeterClipWarningIntensity(0)).toBe(0);
    expect(getMeterClipWarningIntensity(-1)).toBe(0);
    expect(getMeterClipWarningIntensity(Number.NaN)).toBe(0);
  });

  it('maps near-peak levels to bounded warning intensity', () => {
    expect(getMeterClipWarningIntensity(dbToLinear(-1))).toBeCloseTo(0, 5);
    expect(getMeterClipWarningIntensity(dbToLinear(-0.5))).toBeCloseTo(0.5, 3);
    expect(getMeterClipWarningIntensity(1)).toBeCloseTo(1, 5);
  });

  it('supports custom threshold and ceiling, including reversed bounds', () => {
    expect(getMeterClipWarningIntensity(dbToLinear(-3), { thresholdDb: -2, ceilingDb: 0 })).toBe(0);
    expect(
      getMeterClipWarningIntensity(dbToLinear(-1), { thresholdDb: 0, ceilingDb: -2 })
    ).toBeCloseTo(0.5, 3);
  });
});
