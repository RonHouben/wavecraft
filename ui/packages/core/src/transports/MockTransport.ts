/**
 * MockTransport - Test double for Transport interface
 *
 * Provides mock responses for testing without real IPC connection.
 */

import type { Transport, NotificationCallback } from './Transport';
import { IpcMethods } from '../ipc/constants';
import type { IpcResponse } from '../types/ipc';

export class MockTransport implements Transport {
  private connected = true;
  private notificationCallback: NotificationCallback | null = null;
  private readonly connectionChangeCallbacks = new Set<(connected: boolean) => void>();

  public isConnected(): boolean {
    return this.connected;
  }

  public onConnectionChange(callback: (connected: boolean) => void): () => void {
    this.connectionChangeCallbacks.add(callback);
    // Fire immediately with current state (fire-on-subscribe)
    callback(this.connected);
    return () => {
      this.connectionChangeCallbacks.delete(callback);
    };
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
    this.connectionChangeCallbacks.clear();
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
    // Emit connection change event
    for (const callback of this.connectionChangeCallbacks) {
      callback(connected);
    }
  }

  /**
   * Get mock result for a given method
   */
  private getMockResult(method: string, _params?: unknown): unknown {
    switch (method) {
      case IpcMethods.GET_PARAMETER:
        return {
          value: 0,
          default: 0,
          min: 0,
          max: 1,
          name: 'Mock Parameter',
        };

      case IpcMethods.GET_ALL_PARAMETERS:
        return {
          parameters: [],
        };

      case IpcMethods.GET_METER_FRAME:
        return {
          frame: {
            peak_l: 0,
            peak_r: 0,
            rms_l: 0,
            rms_r: 0,
            timestamp: Date.now(),
          },
        };

      case IpcMethods.GET_OSCILLOSCOPE_FRAME:
        return {
          frame: {
            points_l: new Array(1024).fill(0),
            points_r: new Array(1024).fill(0),
            sample_rate: 44100,
            timestamp: Date.now(),
            no_signal: true,
            trigger_mode: 'risingZeroCrossing',
          },
        };

      case IpcMethods.REQUEST_RESIZE:
        return { accepted: true };

      case IpcMethods.SET_PARAMETER:
        return {};

      case IpcMethods.PING:
        return {};

      default:
        // Unknown methods return empty object
        return {};
    }
  }
}
