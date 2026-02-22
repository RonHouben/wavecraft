/**
 * ConnectionStatus Component Tests
 */

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ConnectionStatus } from './ConnectionStatus';

describe('ConnectionStatus', () => {
  it('always renders both transport and audio status badges', () => {
    render(
      <ConnectionStatus
        connected
        transport="websocket"
        phase="runningFullDuplex"
        diagnostic={undefined}
        isReady
        isDegraded={false}
      />
    );

    expect(screen.getByTestId('connection-status')).toBeInTheDocument();
    expect(screen.getByTestId('audio-status')).toBeInTheDocument();
    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('Audio: running (full duplex)')).toBeInTheDocument();
  });

  it('shows degraded diagnostic code when audio is degraded', () => {
    render(
      <ConnectionStatus
        connected
        transport="websocket"
        phase="degraded"
        diagnostic={{ code: 'noOutputDevice', message: 'No output device available' }}
        isReady={false}
        isDegraded
      />
    );

    expect(screen.getByText('Audio: degraded')).toBeInTheDocument();
    expect(screen.getByText('(noOutputDevice)')).toBeInTheDocument();
  });

  it('shows connecting and initializing labels while startup is in progress', () => {
    render(
      <ConnectionStatus
        connected={false}
        transport="websocket"
        phase="initializing"
        diagnostic={undefined}
        isReady={false}
        isDegraded={false}
      />
    );

    expect(screen.getByText('Connecting...')).toBeInTheDocument();
    expect(screen.getByText('Audio: initializing')).toBeInTheDocument();
  });
});
