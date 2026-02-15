/**
 * ConnectionStatus - Visual indicators for IPC transport and runtime audio status
 *
 * This component intentionally always subscribes to runtime audio status so
 * applications do not need any opt-in hook to expose audio readiness.
 */

import React from 'react';
import { useAudioStatus, useConnectionStatus } from '@wavecraft/core';

export function ConnectionStatus(): React.JSX.Element {
  const { connected, transport } = useConnectionStatus();
  const { phase, isReady, isDegraded, diagnostic } = useAudioStatus();

  const transportLabel = connected ? 'Connected' : 'Connecting...';
  const transportTone = connected
    ? 'bg-green-900/30 text-green-400'
    : 'bg-yellow-900/30 text-yellow-400';
  const transportDotTone = connected ? 'bg-green-400' : 'bg-yellow-400';

  let audioLabel = 'Audio: waiting for status';
  let audioTone = 'bg-yellow-900/30 text-yellow-400';
  let audioDotTone = 'bg-yellow-400';

  if (phase === 'runningFullDuplex') {
    audioLabel = 'Audio: running (full duplex)';
    audioTone = 'bg-green-900/30 text-green-400';
    audioDotTone = 'bg-green-400';
  } else if (phase === 'runningInputOnly') {
    audioLabel = 'Audio: running (input only)';
    audioTone = 'bg-green-900/30 text-green-400';
    audioDotTone = 'bg-green-400';
  } else if (phase === 'initializing') {
    audioLabel = 'Audio: initializing';
  } else if (phase === 'disabled') {
    audioLabel = 'Audio: disabled';
    audioTone = 'bg-red-900/30 text-red-400';
    audioDotTone = 'bg-red-400';
  } else if (phase === 'degraded') {
    audioLabel = 'Audio: degraded';
    audioTone = 'bg-orange-900/30 text-orange-400';
    audioDotTone = 'bg-orange-400';
  } else if (phase === 'failed') {
    audioLabel = 'Audio: failed';
    audioTone = 'bg-red-900/30 text-red-400';
    audioDotTone = 'bg-red-400';
  }

  const audioTitle = diagnostic?.message ?? undefined;

  return (
    <div className="flex items-center gap-2">
      <div
        data-testid="connection-status"
        className={`flex items-center gap-2 rounded px-3 py-1.5 text-sm ${transportTone}`}
      >
        <div className={`h-2 w-2 rounded-full ${transportDotTone}`} />
        <span>{transportLabel}</span>
        <span className="text-xs opacity-70">({transport})</span>
      </div>

      <div
        data-testid="audio-status"
        className={`flex items-center gap-2 rounded px-3 py-1.5 text-sm ${audioTone}`}
        title={audioTitle}
      >
        <div className={`h-2 w-2 rounded-full ${audioDotTone}`} />
        <span>{audioLabel}</span>
        {isReady && <span className="text-xs opacity-70">(ready)</span>}
        {isDegraded && diagnostic?.code && (
          <span className="text-xs opacity-70">({diagnostic.code})</span>
        )}
      </div>
    </div>
  );
}
