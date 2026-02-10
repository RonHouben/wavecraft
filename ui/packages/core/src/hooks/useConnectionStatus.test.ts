/**
 * Tests for useConnectionStatus hook
 *
 * Tests event-based connection status updates
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useConnectionStatus } from './useConnectionStatus';
import { IpcBridge } from '../ipc/IpcBridge';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';

describe('useConnectionStatus', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // Reset singleton state
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;

    // Create mock transport
    mockTransport = new MockTransport();

    // Spy on getTransport to return our mock
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);

    // Mock environment as WebSocket (not native)
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  // T17: Event-based status update (connected)
  it('should return connected status when transport is connected', async () => {
    mockTransport.setConnected(true);

    const { result } = renderHook(() => useConnectionStatus());

    await waitFor(() => {
      expect(result.current).toEqual({
        connected: true,
        transport: 'websocket',
      });
    });
  });

  // T18: Disconnect event
  it('should update status when transport disconnects', async () => {
    mockTransport.setConnected(true);

    const { result } = renderHook(() => useConnectionStatus());

    await waitFor(() => {
      expect(result.current.connected).toBe(true);
    });

    // Simulate disconnect
    mockTransport.setConnected(false);

    await waitFor(() => {
      expect(result.current).toEqual({
        connected: false,
        transport: 'none',
      });
    });
  });

  // T19: Native environment
  it('should return native transport type in WebView environment', async () => {
    // Mock environment as native
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(true);

    mockTransport.setConnected(true);

    const { result } = renderHook(() => useConnectionStatus());

    await waitFor(() => {
      expect(result.current).toEqual({
        connected: true,
        transport: 'native',
      });
    });
  });

  // T20: Cleanup on unmount
  it('should unsubscribe from connection changes on unmount', async () => {
    mockTransport.setConnected(true);

    const { unmount } = renderHook(() => useConnectionStatus());

    // Unmount should call unsubscribe
    unmount();

    // Verify callbacks were cleaned up (no errors thrown)
    expect(() => mockTransport.setConnected(false)).not.toThrow();
  });

  it('should handle rapid connection state changes', async () => {
    mockTransport.setConnected(false);

    const { result } = renderHook(() => useConnectionStatus());

    // Rapidly toggle connection
    mockTransport.setConnected(true);
    mockTransport.setConnected(false);
    mockTransport.setConnected(true);

    await waitFor(() => {
      expect(result.current.connected).toBe(true);
    });
  });
});
