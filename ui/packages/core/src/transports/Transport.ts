/**
 * Transport interface for IPC communication
 *
 * Provides an abstraction layer for different transport mechanisms
 * (native WKWebView, WebSocket) with consistent request/notification handling.
 */

/**
 * Callback for handling incoming notifications from the engine
 */
export type NotificationCallback = (notification: string) => void;

/**
 * Transport abstraction for IPC communication
 *
 * Implementations handle the low-level details of sending requests,
 * receiving responses, and dispatching notifications.
 */
export interface Transport {
  /**
   * Send a JSON-RPC request and wait for the response
   *
   * @param request - JSON-RPC request string
   * @returns Promise resolving to JSON-RPC response string
   * @throws Error if transport is not connected or request fails
   */
  send(request: string): Promise<string>;

  /**
   * Register a callback for incoming notifications from the engine
   *
   * @param callback - Function called when a notification arrives
   * @returns Cleanup function to remove the callback
   */
  onNotification(callback: NotificationCallback): () => void;

  /**
   * Check if the transport is currently connected
   *
   * @returns true if transport can send/receive messages
   */
  isConnected(): boolean;

  /**
   * Subscribe to connection state changes
   *
   * Callback fires immediately with current state on subscribe (fire-on-subscribe pattern),
   * then on every subsequent state transition.
   *
   * Optional: if not implemented, IpcBridge falls back to polling.
   *
   * @param callback - Receives true when connected, false when disconnected
   * @returns Cleanup function to remove the listener
   */
  onConnectionChange?(callback: (connected: boolean) => void): () => void;

  /**
   * Clean up resources (close connections, remove listeners)
   *
   * Should be called when the transport is no longer needed.
   */
  dispose(): void;
}
