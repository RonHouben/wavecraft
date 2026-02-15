import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import {
  __resetRegisteredProcessorsForTests,
  registerAvailableProcessors,
} from '../processors/registry';
import { useAvailableProcessors } from './useAvailableProcessors';

afterEach(() => {
  __resetRegisteredProcessorsForTests();
});

describe('useAvailableProcessors', () => {
  it('returns sorted registered processor IDs', () => {
    registerAvailableProcessors(['output_gain', 'oscillator', 'input_gain']);

    const { result } = renderHook(() => useAvailableProcessors());

    expect(result.current).toEqual(['input_gain', 'oscillator', 'output_gain']);
  });

  it('returns empty list when nothing is registered', () => {
    const { result } = renderHook(() => useAvailableProcessors());

    expect(result.current).toEqual([]);
  });
});
