import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Oscilloscope } from './Oscilloscope';

const mockUseConnectionStatus = vi.hoisted(() => vi.fn());
const mockUseOscilloscopeFrame = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useConnectionStatus: mockUseConnectionStatus,
  useOscilloscopeFrame: mockUseOscilloscopeFrame,
}));

describe('Oscilloscope', () => {
  beforeEach(() => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fillStyle: '#000',
      strokeStyle: '#000',
      lineWidth: 1,
    } as unknown as CanvasRenderingContext2D);

    mockUseConnectionStatus.mockReturnValue({ connected: true, transport: 'websocket' });
    mockUseOscilloscopeFrame.mockReturnValue({
      points_l: new Array(1024).fill(0).map((_, idx) => Math.sin((idx / 1024) * Math.PI * 2)),
      points_r: new Array(1024).fill(0).map((_, idx) => Math.cos((idx / 1024) * Math.PI * 2)),
      sample_rate: 44100,
      timestamp: Date.now(),
      no_signal: false,
      trigger_mode: 'risingZeroCrossing',
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('defaults to overlay channel view', () => {
    render(<Oscilloscope />);
    expect(screen.getByTestId('osc-channel-view')).toHaveValue('overlay');
  });

  it('supports channel view switching', () => {
    render(<Oscilloscope />);

    const select = screen.getByTestId('osc-channel-view');
    fireEvent.change(select, { target: { value: 'left' } });
    expect(select).toHaveValue('left');

    fireEvent.change(select, { target: { value: 'right' } });
    expect(select).toHaveValue('right');
  });

  it('defaults trigger mode control to rising zero-crossing', () => {
    render(<Oscilloscope />);
    expect(screen.getByTestId('osc-trigger-mode')).toHaveValue('risingZeroCrossing');
  });

  it('shows no-signal label when frame reports no signal', () => {
    mockUseOscilloscopeFrame.mockReturnValue({
      points_l: new Array(1024).fill(0),
      points_r: new Array(1024).fill(0),
      sample_rate: 44100,
      timestamp: Date.now(),
      no_signal: true,
      trigger_mode: 'risingZeroCrossing',
    });

    render(<Oscilloscope />);

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

    const { unmount } = render(<Oscilloscope />);
    unmount();

    expect(rafSpy).toHaveBeenCalled();
    expect(cancelSpy).toHaveBeenCalled();

    rafSpy.mockRestore();
    cancelSpy.mockRestore();
  });
});
