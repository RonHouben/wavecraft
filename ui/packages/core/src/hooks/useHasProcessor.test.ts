import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import {
  __resetRegisteredProcessorsForTests,
  registerAvailableProcessors,
} from '../processors/registry';
import { useHasProcessor } from './useHasProcessor';

afterEach(() => {
  __resetRegisteredProcessorsForTests();
});

describe('useHasProcessor', () => {
  it('returns true for registered processor id', () => {
    registerAvailableProcessors(['oscillator', 'output_gain']);

    const { result } = renderHook(() => useHasProcessor('oscillator'));

    expect(result.current).toBe(true);
  });

  it('returns false for unknown processor id', () => {
    registerAvailableProcessors(['oscillator']);

    const { result } = renderHook(() => useHasProcessor('example_processor'));

    expect(result.current).toBe(false);
  });
});
