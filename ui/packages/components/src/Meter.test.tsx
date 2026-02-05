/**
 * Meter Component Tests
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Meter } from './Meter';
import { setMockMeterFrame } from '@test/mocks/ipc';

// Mock the IPC module
vi.mock('@wavecraft/core', () => import('@test/mocks/ipc'));

describe('Meter', () => {
  beforeEach(() => {
    // Initialize with silent signal
    setMockMeterFrame({
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      timestamp: 0,
    });
  });

  it('renders meter component', () => {
    render(<Meter />);
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });

  it('displays channel labels', () => {
    render(<Meter />);
    expect(screen.getByText('L')).toBeInTheDocument();
    expect(screen.getByText('R')).toBeInTheDocument();
  });

  it('renders with peak level data', () => {
    setMockMeterFrame({
      peak_l: 0.5,
      peak_r: 0.5,
      rms_l: 0.3,
      rms_r: 0.3,
      timestamp: Date.now(),
    });

    render(<Meter />);

    // Component should render meter bars
    expect(screen.getByText('Levels')).toBeInTheDocument();
    expect(screen.getByText('L')).toBeInTheDocument();
    expect(screen.getByText('R')).toBeInTheDocument();
  });

  it('renders with maximum level data', () => {
    setMockMeterFrame({
      peak_l: 1,
      peak_r: 1,
      rms_l: 0.9,
      rms_r: 0.9,
      timestamp: Date.now(),
    });

    render(<Meter />);

    // Component renders successfully
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });
});
