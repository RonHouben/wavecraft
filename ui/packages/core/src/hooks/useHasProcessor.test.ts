import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import {
  __resetRegisteredProcessorsForTests,
  registerAvailableProcessors,
} from '../processors/registry';
import { useHasProcessorInSignalChain } from './useHasProcessor';

afterEach(() => {
  __resetRegisteredProcessorsForTests();
});

describe('useHasProcessorInSignalChain', () => {
  it('returns true for registered processor id', () => {
    registerAvailableProcessors(['oscillator', 'output_gain']);

    const { result } = renderHook(() => useHasProcessorInSignalChain('oscillator'));

    expect(result.current).toBe(true);
  });

  it('returns false for unknown processor id', () => {
    registerAvailableProcessors(['oscillator']);

    const { result } = renderHook(() => useHasProcessorInSignalChain('example_processor'));

    expect(result.current).toBe(false);
  });
});
