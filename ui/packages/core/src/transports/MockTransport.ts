/**
 * MockTransport - Test double for Transport interface
 *
 * Provides mock responses for testing without real IPC connection.
 */

import type { Transport, NotificationCallback } from './Transport';
import type { IpcResponse } from '../types/ipc';

export class MockTransport implements Transport {
  private connected = true;
  private notificationCallback: NotificationCallback | null = null;

  public isConnected(): boolean {
    return this.connected;
  }

  public async send(requestJson: string): Promise<string> {
    const request = JSON.parse(requestJson);

    // Generate mock responses based on method
    const response: IpcResponse = {
      jsonrpc: '2.0',
      id: request.id,
      result: this.getMockResult(request.method, request.params),
    };

    return JSON.stringify(response);
  }

  public onNotification(callback: NotificationCallback): () => void {
    this.notificationCallback = callback;
    return () => {
      this.notificationCallback = null;
    };
  }

  public close(): void {
    this.connected = false;
  }

  public dispose(): void {
    this.close();
    this.notificationCallback = null;
  }

  /**
   * Simulate sending a notification from the backend
   */
  public simulateNotification(notification: unknown): void {
    if (this.notificationCallback) {
      this.notificationCallback(JSON.stringify(notification));
    }
  }

  /**
   * Set connection status (for testing connection handling)
   */
  public setConnected(connected: boolean): void {
    this.connected = connected;
  }

  /**
   * Get mock result for a given method
   */
  private getMockResult(method: string, _params?: unknown): unknown {
    switch (method) {
      case 'getParameter':
        return {
          value: 0,
          default: 0,
          min: 0,
          max: 1,
          name: 'Mock Parameter',
        };

      case 'getMeterFrame':
        return {
          frame: {
            peak_l: 0,
            peak_r: 0,
            rms_l: 0,
            rms_r: 0,
            timestamp: Date.now(),
          },
        };

      case 'requestResize':
        return { accepted: true };

      case 'setParameter':
        return {};

      default:
        // Unknown methods return empty object
        return {};
    }
  }
}
