import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { IpcBridge } from '../ipc/IpcBridge';
import { SignalChainOrderClient } from '../ipc/SignalChainOrderClient';
import * as transportsModule from '../transports';
import { MockTransport } from '../transports/MockTransport';
import * as environmentModule from '../utils/environment';
import { useSignalChainOrder } from './useSignalChainOrder';

describe('useSignalChainOrder', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (SignalChainOrderClient as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('returns fetched signal chain order when connected', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(SignalChainOrderClient.prototype, 'getSignalChainOrder').mockResolvedValue([
      { id: 'test_tone', type: 'processor' },
      { id: 'oscilloscope_tap', type: 'tap' },
    ]);

    const { result } = renderHook(() => useSignalChainOrder());

    await waitFor(() => {
      expect(result.current.order).toEqual([
        { id: 'test_tone', type: 'processor' },
        { id: 'oscilloscope_tap', type: 'tap' },
      ]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  it('waits for transport connection and fetches order without surfacing an initial error', async () => {
    mockTransport.setConnected(false);

    const getSignalChainOrder = vi
      .spyOn(SignalChainOrderClient.prototype, 'getSignalChainOrder')
      .mockResolvedValue([
        { id: 'input_trim', type: 'processor' },
        { id: 'test_tone', type: 'processor' },
      ]);

    const { result } = renderHook(() => useSignalChainOrder());

    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(true);
    expect(getSignalChainOrder).not.toHaveBeenCalled();

    await act(async () => {
      mockTransport.setConnected(true);
    });

    await waitFor(() => {
      expect(result.current.order).toEqual([
        { id: 'input_trim', type: 'processor' },
        { id: 'test_tone', type: 'processor' },
      ]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    expect(getSignalChainOrder).toHaveBeenCalledTimes(1);
  });
});
