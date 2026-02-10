/**
 * Tests for useAllParameters hook
 *
 * Tests connection-aware parameter loading with state machine
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useAllParameters } from './useAllParameters';
import { IpcBridge } from '../ipc/IpcBridge';
import { ParameterClient } from '../ipc/ParameterClient';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';
import type { ParameterInfo } from '../types/parameters';

// Mock parameters
const mockParams: ParameterInfo[] = [
  {
    id: 'gain',
    name: 'Gain',
    type: 'float',
    value: 0.5,
    default: 0.5,
    unit: 'dB',
  },
  {
    id: 'drive',
    name: 'Drive',
    type: 'float',
    value: 0.3,
    default: 0.3,
    unit: '',
  },
];

describe('useAllParameters', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // Reset singleton states
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (ParameterClient as any).instance = null;

    // Create mock transport
    mockTransport = new MockTransport();

    // Spy on getTransport to return our mock
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);

    // Mock environment as WebSocket (not native)
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);

    // Use fake timers for timeout testing
    vi.useFakeTimers();
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  // T1: Mount when already connected
  it('should load parameters immediately when already connected', async () => {
    mockTransport.setConnected(true);

    // Mock getAllParameters response
    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    // Initially loading
    expect(result.current.isLoading).toBe(true);
    expect(result.current.params).toEqual([]);
    expect(result.current.error).toBeNull();

    // Wait for parameters to load
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.params).toEqual(mockParams);
    expect(result.current.error).toBeNull();
  });

  // T2: Mount disconnected → connect after 500ms
  it('should remain loading until connection establishes', async () => {
    mockTransport.setConnected(false);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    // Should be loading while disconnected
    expect(result.current.isLoading).toBe(true);
    expect(result.current.params).toEqual([]);
    expect(result.current.error).toBeNull();

    // Advance time and connect
    await act(async () => {
      vi.advanceTimersByTime(500);
      mockTransport.setConnected(true);
    });

    // Wait for parameters to load
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.params).toEqual(mockParams);
    expect(result.current.error).toBeNull();
  });

  // T3: Mount disconnected → never connects (timeout)
  it('should show timeout error after 15 seconds', async () => {
    mockTransport.setConnected(false);

    const { result } = renderHook(() => useAllParameters());

    expect(result.current.isLoading).toBe(true);
    expect(result.current.error).toBeNull();

    // Advance time to trigger timeout
    await act(async () => {
      vi.advanceTimersByTime(15000);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).not.toBeNull();
    expect(result.current.error?.message).toContain('wavecraft start');
    expect(result.current.params).toEqual([]);
  });

  // T4: Reconnection auto-refetch
  it('should automatically refetch parameters on reconnection', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    // Wait for initial load
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(getAllSpy).toHaveBeenCalledTimes(1);

    // Disconnect
    await act(async () => {
      mockTransport.setConnected(false);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });

    // Reconnect - should trigger refetch
    await act(async () => {
      mockTransport.setConnected(true);
    });

    await waitFor(() => {
      expect(getAllSpy).toHaveBeenCalledTimes(2);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });

  // T5: Duplicate fetch prevention
  it('should prevent concurrent fetches', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    const getAllSpy = vi
      .spyOn(client, 'getAllParameters')
      .mockImplementation(() => new Promise((resolve) => setTimeout(() => resolve(mockParams), 100)));

    const { result } = renderHook(() => useAllParameters());

    // Trigger multiple connection events rapidly
    await act(async () => {
      mockTransport.setConnected(false);
      mockTransport.setConnected(true);
      mockTransport.setConnected(false);
      mockTransport.setConnected(true);
    });

    // Should only have one in-flight fetch
    await act(async () => {
      vi.advanceTimersByTime(100);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // Should have called at most 2 times (initial + one refetch)
    expect(getAllSpy.mock.calls.length).toBeLessThanOrEqual(2);
  });

  // T6: Cleanup on unmount during WAITING
  it('should clean up when unmounted before connection', async () => {
    mockTransport.setConnected(false);

    const { unmount } = renderHook(() => useAllParameters());

    // Unmount immediately
    unmount();

    // Connect after unmount - should not cause errors
    await act(async () => {
      mockTransport.setConnected(true);
    });

    // No state updates should occur (verified by lack of errors)
  });

  // T7: Cleanup on unmount during FETCH
  it('should clean up when unmounted during fetch', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockImplementation(
      () => new Promise((resolve) => setTimeout(() => resolve(mockParams), 500))
    );

    const { unmount } = renderHook(() => useAllParameters());

    // Unmount during fetch
    await act(async () => {
      vi.advanceTimersByTime(100);
    });

    unmount();

    // Advance time to complete fetch - should not cause errors
    await act(async () => {
      vi.advanceTimersByTime(500);
    });
  });

  // T9: Fetch fails (connected, server error)
  it('should retry 3 times with backoff then show error', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockRejectedValue(new Error('Server error'));

    const { result } = renderHook(() => useAllParameters());

    // Fast-forward through all retries
    await act(async () => {
      vi.advanceTimersByTime(500); // First attempt
      vi.advanceTimersByTime(500); // Retry 1
      vi.advanceTimersByTime(1000); // Retry 2
      vi.advanceTimersByTime(2000); // Retry 3
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // Should have tried 4 times total (initial + 3 retries)
    expect(getAllSpy).toHaveBeenCalledTimes(4);

    expect(result.current.error).not.toBeNull();
    expect(result.current.error?.message).toContain('4 attempts');
  });

  // T10: Transport disconnects mid-fetch
  it('should bail out silently if transport disconnects during fetch', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockImplementation(
      async () => {
        // Disconnect mid-fetch
        mockTransport.setConnected(false);
        throw new Error('Connection lost');
      }
    );

    const { result } = renderHook(() => useAllParameters());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });

    // Should not show error - stays loading
    expect(result.current.error).toBeNull();
  });

  // T11: Native transport (always connected)
  it('should fetch immediately in native mode', async () => {
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(true);
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    const getAllSpy = vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    // Should fetch immediately
    await waitFor(() => {
      expect(getAllSpy).toHaveBeenCalled();
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.params).toEqual(mockParams);
  });

  // T12: reload() while disconnected
  it('should set loading state when reload called while disconnected', async () => {
    mockTransport.setConnected(false);

    const { result } = renderHook(() => useAllParameters());

    expect(result.current.isLoading).toBe(true);

    // Call reload manually
    await act(async () => {
      await result.current.reload();
    });

    // Should still be loading
    expect(result.current.isLoading).toBe(true);
    expect(result.current.error).toBeNull();
  });

  // T13: Error message content (timeout)
  it('should include helpful message in timeout error', async () => {
    mockTransport.setConnected(false);

    const { result } = renderHook(() => useAllParameters());

    await act(async () => {
      vi.advanceTimersByTime(15000);
    });

    await waitFor(() => {
      expect(result.current.error).not.toBeNull();
    });

    expect(result.current.error?.message).toContain('wavecraft start');
    expect(result.current.error?.message).toContain('15 seconds');
  });

  // T14: Error message content (fetch failure)
  it('should include attempt count in fetch failure error', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockRejectedValue(new Error('Fetch failed'));

    const { result } = renderHook(() => useAllParameters());

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    await waitFor(() => {
      expect(result.current.error).not.toBeNull();
    });

    expect(result.current.error?.message).toContain('4 attempts');
    expect(result.current.error?.message).toContain('Fetch failed');
  });

  // T15: Parameter change notification
  it('should update parameter value when notification arrives', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    await waitFor(() => {
      expect(result.current.params).toEqual(mockParams);
    });

    // Simulate parameter change notification
    const onParamChangedCallback = (client.onParameterChanged as unknown as any).mock.calls[0][0];
    await act(async () => {
      onParamChangedCallback('gain', 0.8);
    });

    expect(result.current.params[0].value).toBe(0.8);
  });

  // T16: reload() clears error state
  it('should clear error state when reload is called', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    const getAllSpy = vi
      .spyOn(client, 'getAllParameters')
      .mockRejectedValueOnce(new Error('First error'))
      .mockResolvedValueOnce(mockParams);

    const { result } = renderHook(() => useAllParameters());

    // Fast-forward through retries to get error
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    await waitFor(() => {
      expect(result.current.error).not.toBeNull();
    });

    // Call reload
    await act(async () => {
      await result.current.reload();
    });

    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.params).toEqual(mockParams);
    expect(getAllSpy.mock.calls.length).toBeGreaterThan(4);
  });
});
