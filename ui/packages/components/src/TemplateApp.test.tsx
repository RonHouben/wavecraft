import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

const mockUseWindowResizeSync = vi.hoisted(() => vi.fn());
const mockUseConnectionStatus = vi.hoisted(() => vi.fn());
const mockUseAudioStatus = vi.hoisted(() => vi.fn());
const mockUseLatencyMonitor = vi.hoisted(() => vi.fn());
const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const mockUseRequestResize = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  WavecraftProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
  useWindowResizeSync: mockUseWindowResizeSync,
  useConnectionStatus: mockUseConnectionStatus,
  useAudioStatus: mockUseAudioStatus,
  useLatencyMonitor: mockUseLatencyMonitor,
  useMeterFrame: mockUseMeterFrame,
  useRequestResize: mockUseRequestResize,
}));

vi.mock('@wavecraft/components', async () => {
  const actual =
    await vi.importActual<typeof import('@wavecraft/components')>('@wavecraft/components');
  return {
    ...actual,
    VersionBadge: () => <div data-testid="version-badge" />,
  };
});

vi.mock('../../../../sdk-template/ui/src/processors/OscilloscopeProcessor', () => ({
  OscilloscopeProcessor: () => <div data-testid="oscilloscope" />,
}));
vi.mock('../../../../sdk-template/ui/src/processors/SmartProcessor', () => ({
  SmartProcessor: ({ id }: { id: string }) => <div data-testid={`processor-${id}`} />,
}));

vi.mock('../../../../sdk-template/ui/src/processors/ExampleProcessor', () => ({
  ExampleProcessor: () => <div data-testid="processor-example" />,
}));

import { App } from '../../../../sdk-template/ui/src/App';

describe('sdk-template App layout', () => {
  it('renders oscillator panel and resize handle', () => {
    mockUseConnectionStatus.mockReturnValue({ connected: true, transport: 'websocket' });
    mockUseAudioStatus.mockReturnValue({
      phase: 'runningFullDuplex',
      isReady: true,
      isDegraded: false,
      diagnostic: undefined,
    });
    mockUseLatencyMonitor.mockReturnValue({ latency: 2, avg: 2, max: 4, count: 8 });
    mockUseMeterFrame.mockReturnValue({ peak_l: 0, peak_r: 0, rms_l: 0, rms_r: 0, timestamp: 0 });
    mockUseRequestResize.mockReturnValue(vi.fn());

    render(<App />);

    expect(screen.getByTestId('processor-oscillator')).toBeInTheDocument();
    expect(screen.getByTestId('processor-input_trim')).toBeInTheDocument();
    expect(screen.getByTestId('processor-tone_filter')).toBeInTheDocument();
    expect(screen.getByTestId('processor-soft_clip')).toBeInTheDocument();
    expect(screen.getByTestId('processor-example')).toBeInTheDocument();
    expect(screen.getByTestId('processor-output_gain')).toBeInTheDocument();
    expect(screen.getByLabelText('Resize window')).toBeInTheDocument();
  });
});
