/**
 * Meter Component Tests
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
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
    frame.peak_l = 0;
    frame.peak_r = 0;
    frame.rms_l = 0;
    frame.rms_r = 0;
    frame.timestamp = 0;
  });

  it('renders meter component', () => {
    render(<Meter connected frame={frame} />);
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });

  it('displays channel labels', () => {
    render(<Meter connected frame={frame} />);
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

    render(<Meter connected frame={loudFrame} />);

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

    render(<Meter connected frame={maxFrame} />);

    // Component renders successfully
    expect(screen.getByText('Levels')).toBeInTheDocument();
  });

  it('applies shared focus-visible classes to clip reset button', () => {
    const clippedFrame = {
      peak_l: 1.1,
      peak_r: 0.2,
      rms_l: 0.9,
      rms_r: 0.1,
      timestamp: Date.now(),
    };

    render(<Meter connected frame={clippedFrame} />);

    const clipButton = screen.getByTestId('meter-clip-button');
    expect(clipButton).toHaveClass('focus-visible:ring-2');
    expect(clipButton).toHaveClass('focus-visible:ring-accent');
  });
});
