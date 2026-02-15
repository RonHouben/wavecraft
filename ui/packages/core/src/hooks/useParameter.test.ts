import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useParameter } from './useParameter';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';
import type { ParameterInfo } from '../types/parameters';

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

  it('loads the parameter when connected', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue([mockGainParameter]);

    const { result } = renderHook(() => useParameter('gain'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.param).toEqual(mockGainParameter);
  });

  it('waits through initial disconnect and recovers automatically on reconnect', async () => {
    mockTransport.setConnected(false);
    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue([mockGainParameter]);

    const { result } = renderHook(() => useParameter('gain'));

    expect(result.current.isLoading).toBe(true);
    expect(result.current.error).toBeNull();
    expect(getAllSpy).toHaveBeenCalledTimes(0);

    await act(async () => {
      mockTransport.setConnected(true);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(getAllSpy).toHaveBeenCalledTimes(1);
    expect(result.current.error).toBeNull();
    expect(result.current.param).toEqual(mockGainParameter);
  });

  it('does not surface transient transport-not-connected errors and clears stale errors after reconnect', async () => {
    mockTransport.setConnected(true);
    const client = ParameterClient.getInstance();

    const getAllSpy = vi
      .spyOn(client, 'getAllParameters')
      .mockImplementationOnce(async () => {
        // Simulate race: hook starts fetch while status was connected,
        // then transport drops before invoke succeeds.
        mockTransport.setConnected(false);
        throw new Error('IpcBridge: Transport not connected');
      })
      .mockResolvedValueOnce([mockGainParameter]);

    const { result } = renderHook(() => useParameter('gain'));

    await waitFor(() => {
      expect(getAllSpy).toHaveBeenCalledTimes(1);
    });

    // Transient error should not become visible permanent UI state.
    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(true);

    await act(async () => {
      mockTransport.setConnected(true);
    });

    await waitFor(() => {
      expect(getAllSpy).toHaveBeenCalledTimes(2);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.param).toEqual(mockGainParameter);
  });
});
