import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockUseLatencyMonitor = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useLatencyMonitor: mockUseLatencyMonitor,
}));

import { LatencyMonitor } from './LatencyMonitor';

describe('LatencyMonitor', () => {
  beforeEach(() => {
    mockUseLatencyMonitor.mockReset();
    mockUseLatencyMonitor.mockReturnValue({ latency: null, avg: 0, max: 0, count: 0 });
  });

  it('renders an elevated card shell with idle placeholders', () => {
    render(<LatencyMonitor />);

    const monitor = screen.getByTestId('latency-monitor');
    expect(monitor).toHaveClass('rounded-xl');
    expect(monitor).toHaveClass('bg-plugin-surface-1');
    expect(monitor).toHaveClass('shadow-panel');

    expect(screen.getByText('IPC Latency')).toBeInTheDocument();
    expect(screen.getByTestId('latency-monitor-status')).toHaveTextContent('Idle');
    expect(screen.getByText('Current')).toBeInTheDocument();
    expect(screen.getByText('Average')).toBeInTheDocument();
    expect(screen.getByText('Max')).toBeInTheDocument();
    expect(screen.getByText('Samples')).toBeInTheDocument();
    expect(screen.getByText('0')).toBeInTheDocument();
    expect(mockUseLatencyMonitor).toHaveBeenCalledWith(1000);
  });

  it('renders excellent status for fast average latency', () => {
    mockUseLatencyMonitor.mockReturnValue({ latency: 2.34, avg: 3.45, max: 4.56, count: 12 });

    render(<LatencyMonitor />);

    const status = screen.getByTestId('latency-monitor-status');
    expect(status).toHaveTextContent('Excellent');
    expect(status).toHaveClass('text-state-success');
    expect(screen.getByText('2.34 ms')).toBeInTheDocument();
    expect(screen.getByText('3.45 ms')).toBeInTheDocument();
    expect(screen.getByText('4.56 ms')).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
  });

  it('renders poor status for slow average latency', () => {
    mockUseLatencyMonitor.mockReturnValue({ latency: 14.11, avg: 12.67, max: 18.2, count: 6 });

    render(<LatencyMonitor />);

    const status = screen.getByTestId('latency-monitor-status');
    expect(status).toHaveTextContent('Poor');
    expect(status).toHaveClass('text-state-danger');
  });
});
