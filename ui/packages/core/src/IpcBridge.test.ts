/**
 * Tests for IpcBridge with mock transport
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { IpcBridge } from './ipc/IpcBridge';
import { MockTransport } from './transports/MockTransport';
import * as transportsModule from './transports';

describe('IpcBridge with MockTransport', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // Reset singleton state
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;

    // Create mock transport
    mockTransport = new MockTransport();

    // Spy on getTransport to return our mock
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
  });

  it('should return mock parameter data', async (): Promise<void> => {
    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke('getParameter', { id: 'test' });

    expect(result).toEqual({
      value: 0,
      default: 0,
      min: 0,
      max: 1,
      name: 'Mock Parameter',
    });
  });

  it('should return mock meter frame', async (): Promise<void> => {
    interface MeterFrameResponse {
      frame: {
        peak_l: number;
        peak_r: number;
        rms_l: number;
        rms_r: number;
        timestamp: number;
      };
    }

    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke<MeterFrameResponse>('getMeterFrame');

    expect(result).toHaveProperty('frame');
    expect(result.frame).toHaveProperty('peak_l', 0);
    expect(result.frame).toHaveProperty('peak_r', 0);
    expect(result.frame).toHaveProperty('rms_l', 0);
    expect(result.frame).toHaveProperty('rms_r', 0);
    expect(result.frame).toHaveProperty('timestamp');
  });

  it('should return accepted resize response', async (): Promise<void> => {
    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke('requestResize', { width: 800, height: 600 });

    expect(result).toEqual({ accepted: true });
  });

  it('should return no-op cleanup function for event listeners', (): void => {
    const bridge = IpcBridge.getInstance();
    const unsubscribe = bridge.on('paramUpdate', (): void => {});

    // Should not throw
    expect(unsubscribe).toBeInstanceOf(Function);
    unsubscribe();
  });

  it('should not throw errors when invoking unknown methods', async (): Promise<void> => {
    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke('unknownMethod', { foo: 'bar' });

    // Should return empty object as default
    expect(result).toEqual({});
  });
});
