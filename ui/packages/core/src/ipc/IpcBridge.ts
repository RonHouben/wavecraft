/**
 * IpcBridge - Low-level IPC communication layer
 *
 * Provides a Promise-based API for sending requests and receiving responses
 * using pluggable transport implementations (NativeTransport, WebSocketTransport).
 */

import type { IpcRequest, IpcResponse, IpcNotification } from '../types';
import { isIpcNotification } from '../types';
import type { Transport } from '../transports';
import { getTransport } from '../transports';
import { logger } from '../logger';

type EventCallback<T> = (data: T) => void;

export class IpcBridge {
  private static instance: IpcBridge | null = null;
  private nextId = 1;
  private readonly eventListeners = new Map<string, Set<EventCallback<unknown>>>();
  private transport: Transport | null = null;
  private isInitialized = false;
  private lastDisconnectWarning = 0;
  private readonly DISCONNECT_WARNING_INTERVAL_MS = 5000; // Log warning max once per 5s

  private constructor() {
    // Lazy initialization on first use
  }

  /**
   * Initialize the IPC bridge (lazy)
   */
  private initialize(): void {
    if (this.isInitialized) {
      return;
    }

    // Get transport (auto-selected based on environment)
    this.transport = getTransport();

    // Subscribe to notifications from transport
    this.transport.onNotification((notificationJson: string) => {
      try {
        const parsed = JSON.parse(notificationJson);
        if (isIpcNotification(parsed)) {
          this.handleNotification(parsed);
        }
      } catch (error) {
        logger.error('Failed to parse notification', { error });
      }
    });

    this.isInitialized = true;
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): IpcBridge {
    IpcBridge.instance ??= new IpcBridge();
    return IpcBridge.instance;
  }

  /**
   * Check if the bridge is connected
   */
  public isConnected(): boolean {
    // Trigger lazy initialization so transport gets created
    this.initialize();
    return this.transport?.isConnected() ?? false;
  }

  /**
   * Invoke a method and wait for response
   */
  public async invoke<TResult>(method: string, params?: unknown): Promise<TResult> {
    // Lazy initialization on first use
    this.initialize();

    if (!this.transport?.isConnected()) {
      // Rate-limit disconnect warnings to avoid console spam
      const now = Date.now();
      if (now - this.lastDisconnectWarning > this.DISCONNECT_WARNING_INTERVAL_MS) {
        logger.warn('Transport not connected, call will fail. Waiting for reconnection...');
        this.lastDisconnectWarning = now;
      }
      throw new Error('IpcBridge: Transport not connected');
    }

    const id = this.nextId++;
    const request: IpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };

    // Serialize request
    const requestJson = JSON.stringify(request);

    // Send via transport (transport handles timeout internally)
    const responseJson = await this.transport.send(requestJson);

    // Parse response
    const response: IpcResponse = JSON.parse(responseJson);

    // Check for error
    if (response.error) {
      throw new Error(`IPC Error ${response.error.code}: ${response.error.message}`);
    }

    return response.result as TResult;
  }

  /**
   * Subscribe to notification events
   */
  public on<T>(event: string, callback: EventCallback<T>): () => void {
    // Lazy initialization on first use
    this.initialize();

    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }

    const listeners = this.eventListeners.get(event);
    if (!listeners) {
      throw new Error(`Event listener set not found for event: ${event}`);
    }
    listeners.add(callback as EventCallback<unknown>);

    // Return unsubscribe function
    return () => {
      listeners.delete(callback as EventCallback<unknown>);
    };
  }

  /**
   * Handle notification and dispatch to listeners
   */
  private handleNotification(notification: IpcNotification): void {
    const listeners = this.eventListeners.get(notification.method);
    if (!listeners || listeners.size === 0) {
      return;
    }

    for (const listener of listeners) {
      try {
        listener(notification.params);
      } catch (error) {
        logger.error('Error in event listener', { event: notification.method, error });
      }
    }
  }
}
