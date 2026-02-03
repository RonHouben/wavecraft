/**
 * WebSocketTransport - Browser WebSocket IPC transport
 *
 * Connects to the standalone dev server over WebSocket for
 * browser-based UI development with real engine communication.
 */

import type { Transport, NotificationCallback } from './Transport';
import type { IpcResponse, IpcNotification, RequestId } from '../types';
import { isIpcResponse, isIpcNotification } from '../types';
import { logger } from '../logger/Logger';

interface PendingRequest {
  resolve: (response: string) => void;
  reject: (error: Error) => void;
  timeoutId: ReturnType<typeof setTimeout>;
}

interface WebSocketTransportOptions {
  /** WebSocket server URL (e.g., ws://127.0.0.1:9000) */
  url: string;
  /** Reconnection delay in milliseconds (default: 1000) */
  reconnectDelayMs?: number;
  /** Maximum reconnection attempts (default: 5, use Infinity for unlimited) */
  maxReconnectAttempts?: number;
}

/**
 * WebSocket transport implementation with automatic reconnection
 *
 * Connects to the standalone dev server for browser-based UI development.
 */
export class WebSocketTransport implements Transport {
  private readonly url: string;
  private readonly reconnectDelayMs: number;
  private readonly maxReconnectAttempts: number;

  private ws: WebSocket | null = null;
  private isConnecting = false;
  private reconnectAttempts = 0;
  private reconnectTimeoutId: ReturnType<typeof setTimeout> | null = null;
  private isDisposed = false;
  private maxAttemptsReached = false; // Flag to stop reconnection after max attempts

  private readonly pendingRequests = new Map<RequestId, PendingRequest>();
  private readonly notificationCallbacks = new Set<NotificationCallback>();

  constructor(options: WebSocketTransportOptions) {
    this.url = options.url;
    this.reconnectDelayMs = options.reconnectDelayMs ?? 1000;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5;

    // Start connection immediately
    this.connect();
  }

  /**
   * Send a JSON-RPC request and wait for response
   */
  async send(request: string): Promise<string> {
    if (!this.isConnected()) {
      throw new Error('WebSocketTransport: Not connected');
    }

    const parsedRequest = JSON.parse(request);
    const id = parsedRequest.id;

    if (id === undefined || id === null) {
      throw new Error('WebSocketTransport: Request must have an id');
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
    if (!this.ws) {
      throw new Error('WebSocketTransport: Connection lost');
    }
    this.ws.send(request);

    return responsePromise;
  }

  /**
   * Register a callback for incoming notifications
   */
  onNotification(callback: NotificationCallback): () => void {
    this.notificationCallbacks.add(callback);

    return () => {
      this.notificationCallbacks.delete(callback);
    };
  }

  /**
   * Check if transport is connected
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Clean up resources and close connection
   */
  dispose(): void {
    this.isDisposed = true;

    // Clear reconnect timer
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }

    // Close WebSocket
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

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
   * Attempt to connect to WebSocket server
   */
  private connect(): void {
    if (this.isDisposed || this.isConnecting || this.isConnected()) {
      return;
    }

    this.isConnecting = true;

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = (): void => {
        this.isConnecting = false;
        this.reconnectAttempts = 0;
        logger.info('WebSocketTransport connected', { url: this.url });
      };

      this.ws.onmessage = (event: MessageEvent): void => {
        this.handleIncomingMessage(event.data);
      };

      this.ws.onerror = (error: Event): void => {
        logger.error('WebSocketTransport connection error', { error });
      };

      this.ws.onclose = (): void => {
        this.isConnecting = false;
        this.ws = null;

        if (!this.isDisposed && !this.maxAttemptsReached) {
          this.scheduleReconnect();
        }
      };
    } catch (error) {
      this.isConnecting = false;
      logger.error('WebSocketTransport failed to create WebSocket', { error, url: this.url });
      this.scheduleReconnect();
    }
  }

  /**
   * Schedule reconnection attempt with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.isDisposed || this.maxAttemptsReached) {
      return;
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.maxAttemptsReached = true;
      logger.error('WebSocketTransport max reconnect attempts reached', {
        maxAttempts: this.maxReconnectAttempts,
      });
      // Close the WebSocket to stop browser reconnection attempts
      if (this.ws) {
        this.ws.close();
        this.ws = null;
      }
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelayMs * Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    logger.debug('WebSocketTransport reconnecting', {
      delayMs: delay,
      attempt: this.reconnectAttempts,
      maxAttempts: this.maxReconnectAttempts,
    });

    this.reconnectTimeoutId = setTimeout(() => {
      this.reconnectTimeoutId = null;
      this.connect();
    }, delay);
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
      logger.error('WebSocketTransport failed to parse incoming message', { error, message });
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
        logger.error('WebSocketTransport notification callback error', {
          error,
          method: notification.method,
        });
      }
    }
  }
}
