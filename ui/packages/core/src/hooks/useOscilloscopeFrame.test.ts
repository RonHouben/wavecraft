/**
 * Tests for useOscilloscopeFrame hook
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useOscilloscopeFrame } from './useOscilloscopeFrame';
import { IpcBridge } from '../ipc/IpcBridge';
import { MockTransport } from '../transports/MockTransport';
import * as transportsModule from '../transports';
import * as environmentModule from '../utils/environment';

describe('useOscilloscopeFrame', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);

    vi.spyOn(globalThis, 'requestAnimationFrame').mockImplementation((callback) => {
      return globalThis.setTimeout(() => callback(0), 0) as unknown as number;
    });
    vi.spyOn(globalThis, 'cancelAnimationFrame').mockImplementation((id: number) => {
      clearTimeout(id);
    });

    vi.spyOn(IpcBridge.prototype, 'invoke').mockResolvedValue({
      frame: {
        points_l: new Array(1024).fill(0),
        points_r: new Array(1024).fill(0),
        sample_rate: 44100,
        timestamp: 1,
        no_signal: true,
        trigger_mode: 'risingZeroCrossing',
      },
    });
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('polls oscilloscope frame data using RAF cadence', async () => {
    const { result } = renderHook(() => useOscilloscopeFrame());

    await waitFor(() => {
      expect(result.current).not.toBeNull();
      expect(result.current?.points_l).toHaveLength(1024);
      expect(result.current?.trigger_mode).toBe('risingZeroCrossing');
    });
  });
});
