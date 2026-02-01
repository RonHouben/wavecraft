/**
 * ConnectionStatus - Visual indicator for IPC transport connection
 *
 * Shows connection status for WebSocket transport in browser mode.
 * Native transport (WKWebView) is always connected.
 */

import React from 'react';
import { useConnectionStatus } from '@vstkit/ipc';

export function ConnectionStatus(): React.JSX.Element {
  const { connected, transport } = useConnectionStatus();

  // Native transport is always connected - no need to show indicator
  if (transport === 'native') {
    return <></>;
  }

  return (
    <div
      className={`flex items-center gap-2 rounded px-3 py-1.5 text-sm ${
        connected ? 'bg-green-900/30 text-green-400' : 'bg-yellow-900/30 text-yellow-400'
      }`}
    >
      <div className={`h-2 w-2 rounded-full ${connected ? 'bg-green-400' : 'bg-yellow-400'}`} />
      <span>{connected ? 'Connected' : 'Connecting...'}</span>
      {transport === 'websocket' && <span className="text-xs opacity-70">(WebSocket)</span>}
    </div>
  );
}
