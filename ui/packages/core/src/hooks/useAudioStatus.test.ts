/**
 * Tests for useAudioStatus hook
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useAudioStatus } from './useAudioStatus';
import { IpcBridge } from '../ipc/IpcBridge';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';

describe('useAudioStatus', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('returns fetched status when connected', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(IpcBridge.prototype, 'getAudioStatus').mockResolvedValue({
      phase: 'runningFullDuplex',
      updated_at_ms: 123,
      sample_rate: 44100,
      buffer_size: 512,
    });

    const { result } = renderHook(() => useAudioStatus());

    await waitFor(() => {
      expect(result.current.phase).toBe('runningFullDuplex');
      expect(result.current.isReady).toBe(true);
      expect(result.current.isDegraded).toBe(false);
    });
  });

  it('surfaces explicit failed status when getAudioStatus fails while connected', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(IpcBridge.prototype, 'getAudioStatus').mockRejectedValue(
      new Error('method not found')
    );

    const { result } = renderHook(() => useAudioStatus());

    await waitFor(() => {
      expect(result.current.phase).toBe('failed');
      expect(result.current.isReady).toBe(false);
      expect(result.current.isDegraded).toBe(true);
      expect(result.current.diagnostic?.code).toBe('unknown');
      expect(result.current.diagnostic?.message).toContain('getAudioStatus failed');
    });
  });

  it('clears status on disconnect', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(IpcBridge.prototype, 'getAudioStatus').mockResolvedValue({
      phase: 'runningInputOnly',
      updated_at_ms: 999,
    });

    const { result } = renderHook(() => useAudioStatus());

    await waitFor(() => {
      expect(result.current.phase).toBe('runningInputOnly');
    });

    await act(async () => {
      mockTransport.setConnected(false);
    });

    await waitFor(() => {
      expect(result.current.phase).toBeNull();
      expect(result.current.status).toBeNull();
    });
  });

  it('tracks audioStatusChanged transitions from initializing to running', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(IpcBridge.prototype, 'getAudioStatus').mockResolvedValue({
      phase: 'initializing',
      updated_at_ms: 1,
    });

    const { result } = renderHook(() => useAudioStatus());

    await waitFor(() => {
      expect(result.current.phase).toBe('initializing');
      expect(result.current.isReady).toBe(false);
    });

    await act(async () => {
      mockTransport.simulateNotification({
        jsonrpc: '2.0',
        method: 'audioStatusChanged',
        params: {
          phase: 'runningFullDuplex',
          updated_at_ms: 2,
          sample_rate: 48000,
          buffer_size: 256,
        },
      });
    });

    await waitFor(() => {
      expect(result.current.phase).toBe('runningFullDuplex');
      expect(result.current.isReady).toBe(true);
      expect(result.current.isDegraded).toBe(false);
      expect(result.current.status?.sample_rate).toBe(48000);
      expect(result.current.status?.buffer_size).toBe(256);
    });
  });

  it('ignores malformed audioStatusChanged payloads and keeps previous status', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(IpcBridge.prototype, 'getAudioStatus').mockResolvedValue({
      phase: 'runningInputOnly',
      updated_at_ms: 10,
    });

    const { result } = renderHook(() => useAudioStatus());

    await waitFor(() => {
      expect(result.current.phase).toBe('runningInputOnly');
      expect(result.current.isReady).toBe(true);
    });

    await act(async () => {
      mockTransport.simulateNotification({
        jsonrpc: '2.0',
        method: 'audioStatusChanged',
        params: {
          phase: 'failed',
          // missing updated_at_ms -> intentionally malformed
          diagnostic: {
            code: 'unknown',
            message: 'this should be ignored',
          },
        },
      });
    });

    await waitFor(() => {
      expect(result.current.phase).toBe('runningInputOnly');
      expect(result.current.isReady).toBe(true);
      expect(result.current.isDegraded).toBe(false);
    });
  });
});
