/**
 * ParameterSlider Component Tests
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ParameterSlider } from './ParameterSlider';
import { setMockParameter } from '../test/mocks/ipc';

// Mock the IPC module
vi.mock('@wavecraft/ipc', () => import('../test/mocks/ipc'));

describe('ParameterSlider', () => {
  beforeEach(() => {
    // Set up a mock parameter before each test
    setMockParameter('gain', {
      name: 'Gain',
      value: 0.5,
      default: 0.5,
      type: 'float',
      unit: '%',
    });
  });

  it('renders with parameter name as label', () => {
    render(<ParameterSlider id="gain" />);
    expect(screen.getByText('Gain')).toBeInTheDocument();
  });

  it('displays current parameter value', () => {
    render(<ParameterSlider id="gain" />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveValue('0.5');
  });

  it('displays formatted value with unit', () => {
    render(<ParameterSlider id="gain" />);
    // Value is 0.5, displayed as 50.0%
    expect(screen.getByText('50.0%')).toBeInTheDocument();
  });

  it('updates value on slider change', async () => {
    render(<ParameterSlider id="gain" />);
    const slider = screen.getByRole('slider');

    // Simulate user dragging slider to 0.8
    fireEvent.change(slider, { target: { value: '0.8' } });

    // Check slider value updated
    expect(slider).toHaveValue('0.8');
  });

  it('handles parameter not found', () => {
    render(<ParameterSlider id="nonexistent" />);
    expect(screen.getByText(/Error:/)).toBeInTheDocument();
    expect(screen.getByText(/Parameter not found/)).toBeInTheDocument();
  });

  it('respects min/max bounds', () => {
    render(<ParameterSlider id="gain" />);
    const slider = screen.getByRole('slider');

    // Check slider has correct bounds
    expect(slider).toHaveAttribute('min', '0');
    expect(slider).toHaveAttribute('max', '1');
    expect(slider).toHaveAttribute('step', '0.001');
  });
});
