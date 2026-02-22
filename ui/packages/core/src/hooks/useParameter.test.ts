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
import { useParameter } from './useParameter';

const mockGainParameter: ParameterInfo = {
  id: 'gain',
  name: 'Gain',
  type: 'float',
  value: 0.5,
  default: 0.5,
  min: 0,
  max: 1,
  unit: 'dB',
};

const mockBypassParameter: ParameterInfo = {
  id: 'input_trim_bypass',
  name: 'Input Trim Bypass',
  type: 'bool',
  value: 0,
  default: 0,
  min: 0,
  max: 1,
};

function providerWrapper({ children }: Readonly<{ children: ReactNode }>) {
  return createElement(WavecraftProvider, null, children);
}

describe('useParameter', () => {
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

  it('loads parameter from shared provider state', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue([mockGainParameter]);

    const { result } = renderHook(() => useParameter('gain'), { wrapper: providerWrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.param).toEqual(mockGainParameter);
  });

  it('normalizes bool values from provider state', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue([mockBypassParameter]);

    const { result } = renderHook(() => useParameter('input_trim_bypass'), {
      wrapper: providerWrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.param?.value).toBe(false);
  });

  it('delegates setValue through provider action', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();

    vi.spyOn(client, 'getAllParameters').mockResolvedValue([mockGainParameter]);
    const setSpy = vi.spyOn(client, 'setParameter').mockResolvedValue();

    const { result } = renderHook(() => useParameter('gain'), { wrapper: providerWrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.setValue(0.25);
    });

    expect(setSpy).toHaveBeenCalledWith('gain', 0.25);
    expect(result.current.param?.value).toBe(0.25);
  });
});
