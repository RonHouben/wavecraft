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
      // @ts-expect-error - Intentionally deleting global property for test isolation.
      // This simulates browser environment where IPC primitives are not injected.
      delete globalThis.__VSTKIT_IPC__;
    }
  });

  it('should detect browser environment when IPC primitives are missing', (): void => {
    // @ts-expect-error - Intentionally deleting global property for test isolation.
    // This simulates browser environment where IPC primitives are not injected.
    delete globalThis.__VSTKIT_IPC__;

    expect(isWebViewEnvironment()).toBe(false);
    expect(isBrowserEnvironment()).toBe(true);
  });

  it('should detect WebView environment when IPC primitives are present', (): void => {
    // Mock IPC primitives
    globalThis.__VSTKIT_IPC__ = {
      postMessage: (): void => {},
      setReceiveCallback: (): void => {},
      onParamUpdate: (): void => {},
    };

    expect(isWebViewEnvironment()).toBe(true);
    expect(isBrowserEnvironment()).toBe(false);
  });
});
