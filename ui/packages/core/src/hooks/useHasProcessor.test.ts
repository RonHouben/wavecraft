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
    registerAvailableProcessors(['test_tone', 'output_gain']);

    const { result } = renderHook(() => useHasProcessorInSignalChain('test_tone'));

    expect(result.current).toBe(true);
  });

  it('returns false for unknown processor id', () => {
    registerAvailableProcessors(['test_tone']);

    const { result } = renderHook(() => useHasProcessorInSignalChain('example_processor'));

    expect(result.current).toBe(false);
  });
});
