import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

const mockUseWindowResizeSync = vi.hoisted(() => vi.fn());
const mockUseConnectionStatus = vi.hoisted(() => vi.fn());
const mockUseAudioStatus = vi.hoisted(() => vi.fn());
const mockUseLatencyMonitor = vi.hoisted(() => vi.fn());
const mockUseMeterFrame = vi.hoisted(() => vi.fn());
const mockUseRequestResize = vi.hoisted(() => vi.fn());
const mockUseParameter = vi.hoisted(() => vi.fn());
const mockUseHasProcessorInSignalChain = vi.hoisted(() => vi.fn());
const mockUseOscilloscopeFrame = vi.hoisted(() => vi.fn());
const mockUseSignalChainOrder = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  WavecraftProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
  useWindowResizeSync: mockUseWindowResizeSync,
  useConnectionStatus: mockUseConnectionStatus,
  useAudioStatus: mockUseAudioStatus,
  useLatencyMonitor: mockUseLatencyMonitor,
  useMeterFrame: mockUseMeterFrame,
  useRequestResize: mockUseRequestResize,
  useParameter: mockUseParameter,
  useHasProcessorInSignalChain: mockUseHasProcessorInSignalChain,
  useOscilloscopeFrame: mockUseOscilloscopeFrame,
  useSignalChainOrder: mockUseSignalChainOrder,
}));

vi.mock('@wavecraft/components', async () => {
  const actual =
    await vi.importActual<typeof import('@wavecraft/components')>('@wavecraft/components');
  return {
    ...actual,
    VersionBadge: () => <div data-testid="version-badge" />,
    TestToneProcessor: () => <div data-testid="processor-test-tone" />,
    GainProcessor: ({ processorId }: { processorId: string }) => (
      <div data-testid={`processor-${processorId}`} />
    ),
    SaturatorProcessor: () => <div data-testid="processor-soft_clip" />,
    ToneFilterProcessor: () => <div data-testid="processor-tone_filter" />,
    PassthroughProcessor: () => <div data-testid="processor-example" />,
    OscilloscopeProcessor: () => <div data-testid="oscilloscope" />,
    // Mock SignalChain to avoid @dnd-kit dual-React instance issues in tests.
    // DnD functionality is tested separately in SignalChain.test.tsx.
    SignalChain: ({ entries }: { entries: Array<{ id: string; component: ReactNode }> }) => (
      <ul role="list" aria-label="Signal chain processor order">
        {entries.map((e) =>
          e.component != null ? (
            <li key={e.id} role="listitem">
              {e.component}
            </li>
          ) : null
        )}
      </ul>
    ),
  };
});

import { App } from '../../../../sdk-template/ui/src/App';

describe('sdk-template App layout', () => {
  it('renders test tone panel and resize handle', () => {
    mockUseHasProcessorInSignalChain.mockReturnValue(true);
    mockUseSignalChainOrder.mockReturnValue({
      order: [],
      setOrder: vi.fn(),
      isLoading: false,
      error: null,
    });
    mockUseParameter.mockReturnValue({
      param: {
        id: 'test_parameter',
        name: 'Test Parameter',
        type: 'float',
        value: 0,
        default: 0,
        min: 0,
        max: 1,
        unit: '',
      },
      setValue: vi.fn(),
    });
    mockUseOscilloscopeFrame.mockReturnValue({
      points_l: new Array(128).fill(0),
      points_r: new Array(128).fill(0),
      sample_rate: 44100,
      timestamp: 0,
      no_signal: true,
      trigger_mode: 'risingZeroCrossing',
    });
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

    expect(screen.getByTestId('processor-test-tone')).toBeInTheDocument();
    expect(screen.getByTestId('processor-input_trim')).toBeInTheDocument();
    expect(screen.getByTestId('processor-tone_filter')).toBeInTheDocument();
    expect(screen.getByTestId('processor-soft_clip')).toBeInTheDocument();
    expect(screen.getByTestId('processor-example')).toBeInTheDocument();
    expect(screen.getByTestId('processor-output_gain')).toBeInTheDocument();
    expect(screen.getByRole('list', { name: 'Signal chain processor order' })).toBeInTheDocument();
    expect(screen.getByLabelText('Resize window')).toBeInTheDocument();
  });
});
