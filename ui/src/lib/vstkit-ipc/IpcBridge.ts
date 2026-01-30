/**
 * IpcBridge - Low-level IPC communication layer
 * 
 * Wraps the injected primitives and provides a Promise-based API
 * for sending requests and receiving responses.
 */

import type {
  IpcRequest,
  IpcResponse,
  IpcNotification,
  RequestId,
} from './types';
import { isIpcResponse, isIpcNotification } from './types';

type EventCallback<T> = (data: T) => void;

export class IpcBridge {
  private static instance: IpcBridge | null = null;
  private nextId = 1;
  private pendingRequests = new Map<
    RequestId,
    {
      resolve: (response: IpcResponse) => void;
      reject: (error: Error) => void;
      timeoutId: ReturnType<typeof setTimeout>;
    }
  >();
  private eventListeners = new Map<string, Set<EventCallback<unknown>>>();
  private primitives: typeof globalThis.__VSTKIT_IPC__;

  private constructor() {
    this.primitives = globalThis.__VSTKIT_IPC__;

    if (!this.primitives) {
      throw new Error(
        'IPC primitives not available. Make sure the app is running in VstKit WebView.'
      );
    }

    // Set up receive callback for responses
    this.primitives.setReceiveCallback((message: string) => {
      this.handleIncomingMessage(message);
    });
    
    // Set up parameter update listener for pushed updates from Rust
    if (this.primitives.onParamUpdate) {
      this.primitives.onParamUpdate((notification: unknown) => {
        // Handle as a notification
        if (isIpcNotification(notification)) {
          this.handleNotification(notification);
        }
      });
    }
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): IpcBridge {
    if (!IpcBridge.instance) {
      IpcBridge.instance = new IpcBridge();
    }
    return IpcBridge.instance;
  }

  /**
   * Invoke a method and wait for response
   */
  public async invoke<TResult>(
    method: string,
    params?: unknown,
    timeoutMs = 5000
  ): Promise<TResult> {
    const id = this.nextId++;
    const request: IpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };

    // Create promise for response
    const responsePromise = new Promise<IpcResponse>((resolve, reject) => {
      // Set timeout
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${method}`));
      }, timeoutMs);

      this.pendingRequests.set(id, { resolve, reject, timeoutId });
    });

    // Send request
    const requestJson = JSON.stringify(request);
    this.primitives!.postMessage(requestJson);

    // Wait for response
    const response = await responsePromise;

    // Check for error
    if (response.error) {
      throw new Error(
        `IPC Error ${response.error.code}: ${response.error.message}`
      );
    }

    return response.result as TResult;
  }

  /**
   * Subscribe to notification events
   */
  public on<T>(event: string, callback: EventCallback<T>): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }

    const listeners = this.eventListeners.get(event)!;
    listeners.add(callback as EventCallback<unknown>);

    // Return unsubscribe function
    return () => {
      listeners.delete(callback as EventCallback<unknown>);
    };
  }

  /**
   * Handle incoming message from Rust
   */
  private handleIncomingMessage(message: string): void {
    try {
      const parsed = JSON.parse(message);

      if (isIpcResponse(parsed)) {
        this.handleResponse(parsed);
      } else if (isIpcNotification(parsed)) {
        this.handleNotification(parsed);
      } else {
        console.warn('[IpcBridge] Unknown message type:', parsed);
      }
    } catch (error) {
      console.error('[IpcBridge] Failed to parse message:', error, message);
    }
  }

  /**
   * Handle response from Rust
   */
  private handleResponse(response: IpcResponse): void {
    const pending = this.pendingRequests.get(response.id);
    if (!pending) {
      console.warn('[IpcBridge] Received response for unknown request:', response.id);
      return;
    }

    clearTimeout(pending.timeoutId);
    this.pendingRequests.delete(response.id);
    pending.resolve(response);
  }

  /**
   * Handle notification from Rust
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
        console.error('[IpcBridge] Error in event listener:', error);
      }
    }
  }
}
