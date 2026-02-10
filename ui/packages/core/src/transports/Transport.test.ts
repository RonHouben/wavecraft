/**
 * Tests for Transport onConnectionChange implementation
 *
 * Tests MockTransport and NativeTransport event emission patterns
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { MockTransport } from './MockTransport';
import { NativeTransport } from './NativeTransport';

describe('Transport.onConnectionChange', () => {
  describe('MockTransport (WebSocket-like behavior)', () => {
    let transport: MockTransport;

    afterEach(() => {
      transport?.dispose();
    });

    // T21: Fire-on-subscribe (disconnected)
    it('should fire callback with false immediately when disconnected', () => {
      transport = new MockTransport();
      transport.setConnected(false);

      const callback = vi.fn();
      transport.onConnectionChange(callback);

      // Should fire immediately with false
      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith(false);
    });

    // T22: Fire-on-subscribe (connected)
    it('should fire callback with true immediately when connected', () => {
      transport = new MockTransport();
      // MockTransport defaults to connected

      const callback = vi.fn();
      transport.onConnectionChange(callback);

      // Should fire immediately with true
      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith(true);
    });

    // T23: Transition connected → disconnected
    it('should fire callback on disconnect', () => {
      transport = new MockTransport();

      const callback = vi.fn();
      transport.onConnectionChange(callback);

      callback.mockClear();

      // Simulate disconnect
      transport.setConnected(false);

      expect(callback).toHaveBeenCalledWith(false);
    });

    // T24: Unsubscribe prevents further calls
    it('should not call callback after unsubscribe', () => {
      transport = new MockTransport();

      const callback = vi.fn();
      const unsubscribe = transport.onConnectionChange(callback);

      callback.mockClear();
      unsubscribe();

      // Simulate disconnect (should not fire callback)
      transport.setConnected(false);

      expect(callback).not.toHaveBeenCalled();
    });

    // T25: Multiple subscribers
    it('should notify all subscribers independently', () => {
      transport = new MockTransport();

      const callback1 = vi.fn();
      const callback2 = vi.fn();
      const callback3 = vi.fn();

      transport.onConnectionChange(callback1);
      transport.onConnectionChange(callback2);
      transport.onConnectionChange(callback3);

      // All should have received initial state
      expect(callback1).toHaveBeenCalledWith(true);
      expect(callback2).toHaveBeenCalledWith(true);
      expect(callback3).toHaveBeenCalledWith(true);

      callback1.mockClear();
      callback2.mockClear();
      callback3.mockClear();

      // Simulate disconnect
      transport.setConnected(false);

      // All should receive disconnect
      expect(callback1).toHaveBeenCalledWith(false);
      expect(callback2).toHaveBeenCalledWith(false);
      expect(callback3).toHaveBeenCalledWith(false);
    });

    // T26: dispose() clears callbacks
    it('should not call callbacks after dispose', () => {
      transport = new MockTransport();

      const callback = vi.fn();
      transport.onConnectionChange(callback);

      callback.mockClear();

      // Dispose transport
      transport.dispose();

      // Callbacks should not fire after dispose
      expect(callback).not.toHaveBeenCalled();
    });
  });

  describe('NativeTransport.onConnectionChange', () => {
    let transport: NativeTransport;

    beforeEach(() => {
      // Mock the native IPC primitives
      globalThis.__WAVECRAFT_IPC__ = {
        postMessage: vi.fn(),
        setReceiveCallback: vi.fn(),
        onParamUpdate: vi.fn(),
        _receive: vi.fn(),
        _onParamUpdate: vi.fn(),
      };
    });

    afterEach(() => {
      transport?.dispose();
      delete globalThis.__WAVECRAFT_IPC__;
    });

    // Native transport should always fire true
    it('should fire callback with true immediately (always connected)', () => {
      transport = new NativeTransport();

      const callback = vi.fn();
      transport.onConnectionChange(callback);

      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith(true);
    });

    // Native transport should never fire again
    it('should never fire callback again after initial subscription', () => {
      transport = new NativeTransport();

      const callback = vi.fn();
      const unsubscribe = transport.onConnectionChange(callback);

      callback.mockClear();

      // No further events expected (native doesn't transition)
      expect(callback).not.toHaveBeenCalled();

      unsubscribe();
    });

    // Unsubscribe is no-op for native
    it('should handle unsubscribe gracefully (no-op)', () => {
      transport = new NativeTransport();

      const callback = vi.fn();
      const unsubscribe = transport.onConnectionChange(callback);

      expect(() => unsubscribe()).not.toThrow();
    });
  });
});
