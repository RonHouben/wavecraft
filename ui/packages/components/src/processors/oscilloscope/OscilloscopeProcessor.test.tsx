import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { OscilloscopeProcessor } from './OscilloscopeProcessor';

const mockUseConnectionStatus = vi.hoisted(() => vi.fn());
const mockUseOscilloscopeFrame = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');
  return {
    ...actual,
    useConnectionStatus: mockUseConnectionStatus,
    useOscilloscopeFrame: mockUseOscilloscopeFrame,
  };
});

vi.mock('../ProcessorCard', () => ({
  ProcessorCard: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

const frame = {
  points_l: new Array(1024).fill(0).map((_, idx) => Math.sin((idx / 1024) * Math.PI * 2)),
  points_r: new Array(1024).fill(0).map((_, idx) => Math.cos((idx / 1024) * Math.PI * 2)),
  sample_rate: 44100,
  timestamp: Date.now(),
  no_signal: false,
  trigger_mode: 'risingZeroCrossing',
};

describe('OscilloscopeProcessor', () => {
  beforeEach(() => {
    mockUseConnectionStatus.mockReturnValue({ connected: true, transport: 'websocket' });
    mockUseOscilloscopeFrame.mockReturnValue(frame);
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      setTransform: vi.fn(),
      fillStyle: '#000',
      strokeStyle: '#000',
      lineWidth: 1,
    } as unknown as CanvasRenderingContext2D);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('defaults to overlay channel view', () => {
    render(<OscilloscopeProcessor />);
    expect(screen.getByTestId('osc-channel-view')).toHaveValue('0');
  });

  it('supports channel view switching', () => {
    render(<OscilloscopeProcessor />);

    const select = screen.getByTestId('osc-channel-view');
    fireEvent.change(select, { target: { value: '1' } });
    expect(select).toHaveValue('1');

    fireEvent.change(select, { target: { value: '2' } });
    expect(select).toHaveValue('2');
  });

  it('defaults trigger mode control to rising zero-crossing', () => {
    render(<OscilloscopeProcessor />);
    expect(screen.getByTestId('osc-trigger-mode')).toHaveValue('0');
  });

  it('applies shared focus-visible classes to select controls', () => {
    render(<OscilloscopeProcessor />);

    expect(screen.getByTestId('osc-channel-view')).toHaveClass('focus-visible:ring-2');
    expect(screen.getByTestId('osc-channel-view')).toHaveClass('focus-visible:ring-accent-light');
    expect(screen.getByTestId('osc-trigger-mode')).toHaveClass('focus-visible:ring-2');
    expect(screen.getByTestId('osc-trigger-mode')).toHaveClass('focus-visible:ring-accent-light');
  });

  it('shows no-signal label when frame reports no signal', () => {
    const noSignalFrame = {
      points_l: new Array(1024).fill(0),
      points_r: new Array(1024).fill(0),
      sample_rate: 44100,
      timestamp: Date.now(),
      no_signal: true,
      trigger_mode: 'risingZeroCrossing',
    };

    mockUseOscilloscopeFrame.mockReturnValue(noSignalFrame);

    render(<OscilloscopeProcessor />);

    expect(screen.getByTestId('osc-no-signal')).toHaveTextContent('No signal');
    expect(screen.getByTestId('oscilloscope-canvas')).toBeInTheDocument();
  });

  it('cleans up requestAnimationFrame loop on unmount', () => {
    const rafSpy = vi
      .spyOn(globalThis, 'requestAnimationFrame')
      .mockImplementation((_callback: FrameRequestCallback) => {
        return 1;
      });
    const cancelSpy = vi.spyOn(globalThis, 'cancelAnimationFrame');

    const { unmount } = render(<OscilloscopeProcessor />);
    unmount();

    expect(rafSpy).toHaveBeenCalled();
    expect(cancelSpy).toHaveBeenCalled();

    rafSpy.mockRestore();
    cancelSpy.mockRestore();
  });
});
