/**
 * Transport Factory
 *
 * Provides automatic transport selection based on runtime environment:
 * - WKWebView: NativeTransport
 * - Browser: WebSocketTransport
 */

import type { Transport } from './Transport';
import { NativeTransport } from './NativeTransport';
import { WebSocketTransport } from './WebSocketTransport';
import { isWebViewEnvironment } from '../environment';

// Export transport types
export type { Transport, NotificationCallback } from './Transport';
export { NativeTransport } from './NativeTransport';
export { WebSocketTransport } from './WebSocketTransport';

// Environment detection at module scope (evaluated once)
const IS_WEBVIEW = isWebViewEnvironment();

/**
 * Singleton transport instance
 */
let transportInstance: Transport | null = null;

/**
 * Get the transport instance (singleton)
 *
 * Automatically selects:
 * - NativeTransport in WKWebView (production)
 * - WebSocketTransport in browser (development)
 *
 * @returns Transport instance
 */
export function getTransport(): Transport {
  if (transportInstance) {
    return transportInstance;
  }

  if (IS_WEBVIEW) {
    // Native WKWebView transport
    transportInstance = new NativeTransport();
  } else {
    // WebSocket transport for browser development
    const wsUrl = import.meta.env.VITE_WS_URL || 'ws://127.0.0.1:9000';
    transportInstance = new WebSocketTransport({ url: wsUrl });
  }

  return transportInstance;
}

/**
 * Check if transport is available
 *
 * @returns true if transport exists and is connected
 */
export function hasTransport(): boolean {
  return transportInstance?.isConnected() ?? false;
}

/**
 * Dispose the current transport (mainly for tests)
 */
export function disposeTransport(): void {
  if (transportInstance) {
    transportInstance.dispose();
    transportInstance = null;
  }
}
