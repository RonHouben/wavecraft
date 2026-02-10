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
  let onParamChangedCallback: ((id: string, value: number) => void) | null = null;

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

    // Mock onParameterChanged to prevent real subscription setup
    // This is critical: without this mock, the hook tries to subscribe via
    // IpcBridge.on(), which initializes the transport and sets up polling
    // intervals that interact badly with fake timers
    onParamChangedCallback = null;
    vi.spyOn(ParameterClient.prototype, 'onParameterChanged').mockImplementation((callback) => {
      onParamChangedCallback = callback;
      return () => {
        onParamChangedCallback = null;
      };
    });

    // NOTE: Don't use fake timers globally - only in specific tests that need them
    // Most tests work fine with real timers and waitFor()
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
    // NOTE: No need for vi.useRealTimers() since we don't use fake timers globally
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

    // Connect (no need for fake timers - just trigger connection event)
    mockTransport.setConnected(true);

    // Wait for parameters to load
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.params).toEqual(mockParams);
    expect(result.current.error).toBeNull();
  });

  // T3: Mount disconnected → never connects (timeout)
  it('should show timeout error after 15 seconds', async () => {
    // Use fake timers for this timeout test
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(false);

      const { result } = renderHook(() => useAllParameters());

      expect(result.current.isLoading).toBe(true);
      expect(result.current.error).toBeNull();

      // Run all timers to trigger timeout
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toContain('wavecraft start');
      expect(result.current.params).toEqual([]);
    } finally {
      vi.useRealTimers();
    }
  }, 10000); // Increase timeout for this test

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

    // Wait for refetch to complete
    await waitFor(() => {
      expect(getAllSpy).toHaveBeenCalledTimes(2);
    }, { timeout: 1000 });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    }, { timeout: 1000 });
  });

  // T5: Duplicate fetch prevention
  it('should prevent concurrent fetches', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    // Use a delayed Promise without setTimeout (avoids timer conflicts)
    const getAllSpy = vi
      .spyOn(client, 'getAllParameters')
      .mockImplementation(() => new Promise((resolve) => {
        // Resolve after a microtask delay
        queueMicrotask(() => resolve(mockParams));
      }));

    const { result } = renderHook(() => useAllParameters());

    // Trigger multiple connection events rapidly
    await act(async () => {
      mockTransport.setConnected(false);
      mockTransport.setConnected(true);
      mockTransport.setConnected(false);
      mockTransport.setConnected(true);
    });

    // Wait for loading to complete
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    }, { timeout: 1000 });

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

    // Unmount during fetch (no need to advance timers - just unmount immediately)
    unmount();

    // No state updates should occur after unmount (verified by lack of errors)
  });

  // T9: Fetch fails (connected, server error)
  it('should retry 3 times with backoff then show error', async () => {
    // Use fake timers for backoff testing
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(true);

      const client = ParameterClient.getInstance();
      const getAllSpy = vi.spyOn(client, 'getAllParameters').mockRejectedValue(new Error('Server error'));

      const { result } = renderHook(() => useAllParameters());

      // Run all timers to complete all retries
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.isLoading).toBe(false);

      // Should have tried 4 times total (initial + 3 retries)
      expect(getAllSpy).toHaveBeenCalledTimes(4);

      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toContain('4 attempts');
    } finally {
      vi.useRealTimers();
    }
  }, 10000); // Increase timeout for this test

  // T10: Transport disconnects mid-fetch
  it('should bail out silently if transport disconnects during fetch', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockImplementation(
      async () => {
        // Disconnect mid-fetch
        await act(async () => {
          mockTransport.setConnected(false);
        });
        throw new Error('Connection lost');
      }
    );

    const { result } = renderHook(() => useAllParameters());

    // Wait for the disconnect to be processed
    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    }, { timeout: 1000 });

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
    }, { timeout: 1000 });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    }, { timeout: 1000 });

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
    // Use fake timers for timeout test
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(false);

      const { result } = renderHook(() => useAllParameters());

      // Run all timers
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toContain('wavecraft start');
      expect(result.current.error?.message).toContain('15 seconds');
    } finally {
      vi.useRealTimers();
    }
  }, 10000); // Increase timeout for this test

  // T14: Error message content (fetch failure)
  it('should include attempt count in fetch failure error', async () => {
    // Use fake timers for retry backoff
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(true);

      const client = ParameterClient.getInstance();
      vi.spyOn(client, 'getAllParameters').mockRejectedValue(new Error('Fetch failed'));

      const { result } = renderHook(() => useAllParameters());

      // Run all timers
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toContain('4 attempts');
      expect(result.current.error?.message).toContain('Fetch failed');
    } finally {
      vi.useRealTimers();
    }
  }, 10000); // Increase timeout for this test

  // T15: Parameter change notification
  it('should update parameter value when notification arrives', async () => {
    mockTransport.setConnected(true);

    const client = ParameterClient.getInstance();
    vi.spyOn(client, 'getAllParameters').mockResolvedValue(mockParams);

    const { result } = renderHook(() => useAllParameters());

    await waitFor(() => {
      expect(result.current.params).toEqual(mockParams);
    });

    // Simulate parameter change notification using captured callback
    expect(onParamChangedCallback).not.toBeNull();
    if (onParamChangedCallback) {
      await act(async () => {
        onParamChangedCallback('gain', 0.8);
      });
    }

    // Wait for the state update to complete
    await waitFor(() => {
      expect(result.current.params[0].value).toBe(0.8);
    }, { timeout: 1000 });
  });

  // T16: reload() clears error state
  it('should clear error state when reload is called', async () => {
    // Use fake timers for retry testing
    vi.useFakeTimers();

    try {
      mockTransport.setConnected(true);

      const client = ParameterClient.getInstance();
      const getAllSpy = vi
        .spyOn(client, 'getAllParameters')
        .mockRejectedValueOnce(new Error('First error'))
        .mockRejectedValueOnce(new Error('First error'))
        .mockRejectedValueOnce(new Error('First error'))
        .mockRejectedValueOnce(new Error('First error'))
        .mockResolvedValueOnce(mockParams);

      const { result } = renderHook(() => useAllParameters());

      // Run all timers to get the error state
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.error).not.toBeNull();
      expect(result.current.isLoading).toBe(false);

      // Call reload
      await act(async () => {
        await result.current.reload();
      });

      expect(result.current.error).toBeNull();
      expect(result.current.isLoading).toBe(false); // Should be false since fetch succeeded
      expect(result.current.params).toEqual(mockParams);
      expect(getAllSpy.mock.calls.length).toBeGreaterThan(4);
    } finally {
      vi.useRealTimers();
    }
  }, 10000); // Increase timeout for this test
});
