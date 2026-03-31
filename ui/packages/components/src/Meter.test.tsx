/**
 * Meter Component Tests
 */

import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseConnectionStatus = vi.hoisted(() => vi.fn());
const mockUseMeterFrame = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useConnectionStatus: mockUseConnectionStatus,
  useMeterFrame: mockUseMeterFrame,
}));

import { Meter } from './Meter';

const frame = {
  peak_l: 0,
  peak_r: 0,
  rms_l: 0,
  rms_r: 0,
  timestamp: 0,
};

describe('Meter', () => {
  beforeEach(() => {
    mockUseConnectionStatus.mockReset();
    mockUseMeterFrame.mockReset();

    frame.peak_l = 0;
    frame.peak_r = 0;
    frame.rms_l = 0;
    frame.rms_r = 0;
    frame.timestamp = 0;

    mockUseConnectionStatus.mockReturnValue({ connected: true, transport: 'websocket' });
    mockUseMeterFrame.mockImplementation(() => frame);
  });

  it('renders meter component', () => {
    render(<Meter />);
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });

  it('uses the elevated processor-style card shell', () => {
    render(<Meter />);

    const meter = screen.getByTestId('meter');
    expect(meter).toHaveClass('rounded-xl');
    expect(meter).toHaveClass('bg-plugin-surface-1');
    expect(meter).toHaveClass('shadow-panel');
  });

  it('displays channel labels', () => {
    render(<Meter />);
    expect(screen.getByText('L')).toBeInTheDocument();
    expect(screen.getByText('R')).toBeInTheDocument();
  });

  it('renders with peak level data', () => {
    const loudFrame = {
      peak_l: 0.5,
      peak_r: 0.5,
      rms_l: 0.3,
      rms_r: 0.3,
      timestamp: Date.now(),
    };

    mockUseMeterFrame.mockReturnValue(loudFrame);

    render(<Meter />);

    // Component should render meter bars
    expect(screen.getByText('Levels')).toBeInTheDocument();
    expect(screen.getByText('L')).toBeInTheDocument();
    expect(screen.getByText('R')).toBeInTheDocument();
  });

  it('renders with maximum level data', () => {
    const maxFrame = {
      peak_l: 1,
      peak_r: 1,
      rms_l: 0.9,
      rms_r: 0.9,
      timestamp: Date.now(),
    };

    mockUseMeterFrame.mockReturnValue(maxFrame);

    render(<Meter />);

    // Component renders successfully
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });

  it('applies shared focus-visible classes to clip reset button', async () => {
    const clippedFrame = {
      peak_l: 1.1,
      peak_r: 0.2,
      rms_l: 0.9,
      rms_r: 0.1,
      timestamp: Date.now(),
    };

    mockUseMeterFrame.mockReturnValue(clippedFrame);

    render(<Meter />);

    const clipButton = await screen.findByTestId('meter-clip-button');
    expect(clipButton).toHaveClass('focus-visible:ring-2');
    expect(clipButton).toHaveClass('focus-visible:ring-accent-light');
  });

  it('renders plugin state badge when provided', () => {
    render(<Meter pluginState="mapped" />);

    expect(screen.getByText('MAP')).toBeInTheDocument();
  });

  it('applies disabled state cue when state is disabled', () => {
    render(<Meter state="disabled" />);

    const meter = screen.getByTestId('meter');
    expect(meter).toHaveAttribute('data-state', 'disabled');
    expect(meter).toHaveClass('cursor-not-allowed');
  });
});
