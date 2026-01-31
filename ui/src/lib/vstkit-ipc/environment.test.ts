/**
 * Tests for environment detection
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { isWebViewEnvironment, isBrowserEnvironment } from './environment';

describe('Environment Detection', () => {
  let originalVstkit: typeof globalThis.__VSTKIT_IPC__;

  beforeEach(() => {
    // Save original state
    originalVstkit = globalThis.__VSTKIT_IPC__;
  });

  afterEach(() => {
    // Restore original state
    if (originalVstkit) {
      globalThis.__VSTKIT_IPC__ = originalVstkit;
    } else {
      delete (globalThis as { __VSTKIT_IPC__?: unknown }).__VSTKIT_IPC__;
    }
  });

  it('should detect browser environment when IPC primitives are missing', (): void => {
    delete (globalThis as { __VSTKIT_IPC__?: unknown }).__VSTKIT_IPC__;

    expect(isWebViewEnvironment()).toBe(false);
    expect(isBrowserEnvironment()).toBe(true);
  });

  it('should detect WebView environment when IPC primitives are present', (): void => {
    // Mock IPC primitives
    globalThis.__VSTKIT_IPC__ = {
      postMessage: (): void => {},
      setReceiveCallback: (_cb: (message: string) => void): void => {},
      onParamUpdate: (_callback: (notification: unknown) => void): (() => void) => {
        return () => {};
      },
      _receive: (): void => {},
    };

    expect(isWebViewEnvironment()).toBe(true);
    expect(isBrowserEnvironment()).toBe(false);
  });
});
