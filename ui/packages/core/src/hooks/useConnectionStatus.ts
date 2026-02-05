/**
 * useConnectionStatus - Monitor transport connection status
 *
 * Provides real-time connection status updates for the IPC transport.
 * Useful for showing connection indicators in the UI.
 */

import { useEffect, useState } from 'react';
import { IpcBridge } from '../ipc/IpcBridge';
import { isWebViewEnvironment } from '../utils/environment';

export type TransportType = 'native' | 'websocket' | 'none';

export interface ConnectionStatus {
  /** Whether transport is connected and ready */
  connected: boolean;
  /** Type of transport being used */
  transport: TransportType;
}

/**
 * Hook to monitor IPC connection status
 *
 * Polls the transport every second to detect connection changes.
 * Native transport is always connected, WebSocket may reconnect.
 *
 * @returns Connection status object
 */
export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>(() => {
    const bridge = IpcBridge.getInstance();
    const connected = bridge.isConnected();

    let transport: TransportType;
    if (isWebViewEnvironment()) {
      transport = 'native';
    } else if (connected) {
      transport = 'websocket';
    } else {
      transport = 'none';
    }

    return { connected, transport };
  });

  useEffect(() => {
    const bridge = IpcBridge.getInstance();

    // Poll connection status every second
    const intervalId = setInterval(() => {
      const connected = bridge.isConnected();

      let transport: TransportType;
      if (isWebViewEnvironment()) {
        transport = 'native';
      } else if (connected) {
        transport = 'websocket';
      } else {
        transport = 'none';
      }

      setStatus((prevStatus) => {
        // Only update if status changed (avoid unnecessary re-renders)
        if (prevStatus.connected !== connected || prevStatus.transport !== transport) {
          return { connected, transport };
        }
        return prevStatus;
      });
    }, 1000);

    // Cleanup interval on unmount
    return (): void => {
      clearInterval(intervalId);
    };
  }, []);

  return status;
}
