/**
 * Tests for IpcBridge browser mode graceful degradation
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { IpcBridge } from './IpcBridge';

describe('IpcBridge Browser Mode', () => {
  let originalVstkit: typeof globalThis.__VSTKIT_IPC__;

  beforeEach(() => {
    // Save original state
    originalVstkit = globalThis.__VSTKIT_IPC__;
    // Ensure we're in browser mode for these tests
    delete globalThis.__VSTKIT_IPC__;
  });

  afterEach(() => {
    // Restore original state
    if (originalVstkit) {
      globalThis.__VSTKIT_IPC__ = originalVstkit;
    } else {
      delete globalThis.__VSTKIT_IPC__;
    }
  });

  it('should return mock parameter data in browser mode', async (): Promise<void> => {
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

  it('should return mock meter frame in browser mode', async (): Promise<void> => {
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

  it('should return accepted resize response in browser mode', async (): Promise<void> => {
    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke('requestResize', { width: 800, height: 600 });

    expect(result).toEqual({ accepted: true });
  });

  it('should return no-op cleanup function for event listeners in browser mode', (): void => {
    const bridge = IpcBridge.getInstance();
    const unsubscribe = bridge.on('paramUpdate', (): void => {});

    // Should not throw
    expect(unsubscribe).toBeInstanceOf(Function);
    unsubscribe();
  });

  it('should not throw errors when invoking unknown methods in browser mode', async (): Promise<void> => {
    const bridge = IpcBridge.getInstance();
    const result = await bridge.invoke('unknownMethod', { foo: 'bar' });

    // Should return empty object as default
    expect(result).toEqual({});
  });
});
