/**
 * NativeTransport - WKWebView IPC transport
 *
 * Wraps the native IPC primitives injected by the Rust engine
 * into a WKWebView. This transport is always connected.
 */

import type { Transport, NotificationCallback } from './Transport';
import type { IpcResponse, IpcNotification, RequestId } from '../types';
import { isIpcResponse, isIpcNotification } from '../types';
import { logger } from '@wavecraft/ipc';

interface PendingRequest {
  resolve: (response: string) => void;
  reject: (error: Error) => void;
  timeoutId: ReturnType<typeof setTimeout>;
}

/**
 * Native WKWebView transport implementation
 *
 * Uses the __WAVECRAFT_IPC__ primitives injected by the Rust engine.
 */
export class NativeTransport implements Transport {
  private readonly pendingRequests = new Map<RequestId, PendingRequest>();
  private readonly notificationCallbacks = new Set<NotificationCallback>();
  private readonly primitives: typeof globalThis.__WAVECRAFT_IPC__;

  constructor() {
    this.primitives = globalThis.__WAVECRAFT_IPC__;

    if (!this.primitives) {
      throw new Error(
        'NativeTransport: __WAVECRAFT_IPC__ primitives not found. ' +
          'Ensure this runs in a WKWebView with injected IPC.'
      );
    }

    // Set up receive callback for responses
    this.primitives.setReceiveCallback((message: string) => {
      this.handleIncomingMessage(message);
    });

    // Set up parameter update listener for pushed updates
    if (this.primitives.onParamUpdate) {
      this.primitives.onParamUpdate((notification: unknown) => {
        if (isIpcNotification(notification)) {
          this.handleNotification(notification);
        }
      });
    }
  }

  /**
   * Send a JSON-RPC request and wait for response
   */
  async send(request: string): Promise<string> {
    if (!this.primitives) {
      throw new Error('NativeTransport: Primitives not available');
    }

    const parsedRequest = JSON.parse(request);
    const id = parsedRequest.id;

    if (id === undefined || id === null) {
      throw new Error('NativeTransport: Request must have an id');
    }

    // Create promise for response
    const responsePromise = new Promise<string>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${parsedRequest.method}`));
      }, 5000); // 5 second timeout

      this.pendingRequests.set(id, { resolve, reject, timeoutId });
    });

    // Send request
    this.primitives.postMessage(request);

    return responsePromise;
  }

  /**
   * Register a callback for incoming notifications
   */
  onNotification(callback: NotificationCallback): () => void {
    this.notificationCallbacks.add(callback);

    // Return cleanup function
    return () => {
      this.notificationCallbacks.delete(callback);
    };
  }

  /**
   * Check if transport is connected (native is always connected)
   */
  isConnected(): boolean {
    return true;
  }

  /**
   * Clean up resources
   */
  dispose(): void {
    // Cancel all pending requests
    for (const [id, { reject, timeoutId }] of this.pendingRequests.entries()) {
      clearTimeout(timeoutId);
      reject(new Error('Transport disposed'));
      this.pendingRequests.delete(id);
    }

    // Clear notification callbacks
    this.notificationCallbacks.clear();
  }

  /**
   * Handle incoming message (response or notification)
   */
  private handleIncomingMessage(message: string): void {
    try {
      const parsed = JSON.parse(message);

      if (isIpcResponse(parsed)) {
        this.handleResponse(parsed);
      } else if (isIpcNotification(parsed)) {
        this.handleNotification(parsed);
      }
    } catch (error) {
      logger.error('NativeTransport failed to parse incoming message', { error, message });
    }
  }

  /**
   * Handle JSON-RPC response
   */
  private handleResponse(response: IpcResponse): void {
    const pending = this.pendingRequests.get(response.id);

    if (pending) {
      clearTimeout(pending.timeoutId);
      this.pendingRequests.delete(response.id);
      pending.resolve(JSON.stringify(response));
    }
  }

  /**
   * Handle notification and dispatch to listeners
   */
  private handleNotification(notification: IpcNotification): void {
    const notificationJson = JSON.stringify(notification);

    for (const callback of this.notificationCallbacks) {
      try {
        callback(notificationJson);
      } catch (error) {
        logger.error('NativeTransport notification callback error', { error });
      }
    }
  }
}
