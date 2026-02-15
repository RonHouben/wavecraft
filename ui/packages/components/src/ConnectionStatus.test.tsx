/**
 * ConnectionStatus Component Tests
 */

import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ConnectionStatus } from './ConnectionStatus';

const useConnectionStatusMock = vi.fn();
const useAudioStatusMock = vi.fn();

vi.mock('@wavecraft/core', () => ({
  useConnectionStatus: (): unknown => useConnectionStatusMock(),
  useAudioStatus: (): unknown => useAudioStatusMock(),
}));

describe('ConnectionStatus', () => {
  beforeEach(() => {
    useConnectionStatusMock.mockReturnValue({
      connected: true,
      transport: 'websocket',
    });

    useAudioStatusMock.mockReturnValue({
      phase: 'runningFullDuplex',
      diagnostic: undefined,
      isReady: true,
      isDegraded: false,
      status: {
        phase: 'runningFullDuplex',
        updated_at_ms: 123,
        sample_rate: 48000,
        buffer_size: 256,
      },
    });
  });

  it('always renders both transport and audio status badges', () => {
    render(<ConnectionStatus />);

    expect(screen.getByTestId('connection-status')).toBeInTheDocument();
    expect(screen.getByTestId('audio-status')).toBeInTheDocument();
    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('Audio: running (full duplex)')).toBeInTheDocument();
  });

  it('shows degraded diagnostic code when audio is degraded', () => {
    useAudioStatusMock.mockReturnValue({
      phase: 'degraded',
      diagnostic: {
        code: 'noOutputDevice',
        message: 'No output device available',
      },
      isReady: false,
      isDegraded: true,
      status: {
        phase: 'degraded',
        updated_at_ms: 321,
      },
    });

    render(<ConnectionStatus />);

    expect(screen.getByText('Audio: degraded')).toBeInTheDocument();
    expect(screen.getByText('(noOutputDevice)')).toBeInTheDocument();
  });

  it('shows connecting and initializing labels while startup is in progress', () => {
    useConnectionStatusMock.mockReturnValue({
      connected: false,
      transport: 'websocket',
    });
    useAudioStatusMock.mockReturnValue({
      phase: 'initializing',
      diagnostic: undefined,
      isReady: false,
      isDegraded: false,
      status: {
        phase: 'initializing',
        updated_at_ms: 999,
      },
    });

    render(<ConnectionStatus />);

    expect(screen.getByText('Connecting...')).toBeInTheDocument();
    expect(screen.getByText('Audio: initializing')).toBeInTheDocument();
  });
});
