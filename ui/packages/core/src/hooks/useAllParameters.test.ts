import { renderHook, waitFor, act } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { WavecraftProvider } from '../context/WavecraftProvider';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';
import type { ParameterInfo } from '../types/parameters';
import { useAllParameters } from './useAllParameters';
import { useParameter } from './useParameter';

const mockParams: ParameterInfo[] = [
  {
    id: 'gain',
    name: 'Gain',
    type: 'float',
    value: 0.5,
    default: 0.5,
    min: 0,
    max: 1,
    unit: 'dB',
  },
  {
    id: 'input_trim_bypass',
    name: 'Input Trim Bypass',
    type: 'bool',
    value: 0,
    default: 0,
    min: 0,
    max: 1,
  },
];

function providerWrapper({ children }: Readonly<{ children: ReactNode }>) {
  return createElement(WavecraftProvider, null, children);
}

describe('useAllParameters', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (ParameterClient as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
    vi.spyOn(ParameterClient.prototype, 'onParameterChanged').mockImplementation(() => () => {});
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('requires WavecraftProvider context', () => {
    expect(() => renderHook(() => useAllParameters())).toThrowError(
      /WavecraftProvider is required/
    );
  });

  it('provides shared parameter state from WavecraftProvider', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters(), { wrapper: providerWrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.params).toHaveLength(2);
    expect(result.current.params[0]?.id).toBe('gain');
    expect(typeof result.current.setParameter).toBe('function');
  });

  it('deduplicates fetches across mixed parameter hooks under one provider', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(
      () => ({ all: useAllParameters(), single: useParameter('gain') }),
      {
        wrapper: providerWrapper,
      }
    );

    await waitFor(() => {
      expect(result.current.all.isLoading).toBe(false);
      expect(result.current.single.isLoading).toBe(false);
    });

    expect(getAllSpy).toHaveBeenCalledTimes(1);
    expect(result.current.single.param?.id).toBe('gain');
  });

  it('updates shared state after setParameter action', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();

    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);
    const setSpy = vi.spyOn(client, 'setParameter').mockResolvedValue();

    const { result } = renderHook(() => useAllParameters(), { wrapper: providerWrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.setParameter('gain', 0.8);
    });

    expect(setSpy).toHaveBeenCalledWith('gain', 0.8);
    const gain = result.current.params.find((param) => param.id === 'gain');
    expect(gain?.value).toBe(0.8);
  });
});
