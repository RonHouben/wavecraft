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
 * Uses event-based notification from IpcBridge for real-time updates.
 * Native transport is always connected, WebSocket may reconnect.
 *
 * @returns Connection status object
 */
export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    transport: 'none',
  });

  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const isNative = isWebViewEnvironment();

    const unsubscribe = bridge.onConnectionChange((connected) => {
      let transport: TransportType;
      if (isNative) {
        transport = 'native';
      } else if (connected) {
        transport = 'websocket';
      } else {
        transport = 'none';
      }

      setStatus((prev) => {
        if (prev.connected !== connected || prev.transport !== transport) {
          return { connected, transport };
        }
        return prev;
      });
    });

    return unsubscribe;
  }, []);

  return status;
}
