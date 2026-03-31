import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { InputSourceClient } from '../ipc/InputSourceClient';
import { IpcBridge } from '../ipc/IpcBridge';
import * as transportsModule from '../transports';
import { MockTransport } from '../transports/MockTransport';
import * as environmentModule from '../utils/environment';
import { useInputSource } from './useInputSource';

describe('useInputSource', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (InputSourceClient as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('returns fetched input source selection when connected', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(InputSourceClient.prototype, 'getInputSource').mockResolvedValue({
      selected: 'hardwareInput',
      available: [
        { id: 'hardwareInput', label: 'Soundcard input' },
        { id: 'testTone', label: 'Test tone' },
      ],
    });

    const { result } = renderHook(() => useInputSource());

    await waitFor(() => {
      expect(result.current.selected).toBe('hardwareInput');
      expect(result.current.available).toHaveLength(2);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  it('tracks inputSourceChanged notifications', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(InputSourceClient.prototype, 'getInputSource').mockResolvedValue({
      selected: 'hardwareInput',
      available: [
        { id: 'hardwareInput', label: 'Soundcard input' },
        { id: 'testTone', label: 'Test tone' },
      ],
    });

    const { result } = renderHook(() => useInputSource());

    await waitFor(() => {
      expect(result.current.selected).toBe('hardwareInput');
    });

    await act(async () => {
      mockTransport.simulateNotification({
        jsonrpc: '2.0',
        method: 'inputSourceChanged',
        params: {
          selected: 'testTone',
        },
      });
    });

    await waitFor(() => {
      expect(result.current.selected).toBe('testTone');
    });
  });

  it('rolls back optimistic selection when setInputSource fails', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(InputSourceClient.prototype, 'getInputSource').mockResolvedValue({
      selected: 'hardwareInput',
      available: [
        { id: 'hardwareInput', label: 'Soundcard input' },
        { id: 'testTone', label: 'Test tone' },
      ],
    });
    vi.spyOn(InputSourceClient.prototype, 'setInputSource').mockRejectedValue(new Error('nope'));

    const { result } = renderHook(() => useInputSource());

    await waitFor(() => {
      expect(result.current.selected).toBe('hardwareInput');
    });

    await expect(result.current.setSelected('testTone')).rejects.toThrow('nope');

    await waitFor(() => {
      expect(result.current.selected).toBe('hardwareInput');
      expect(result.current.error?.message).toContain('nope');
    });
  });
});
