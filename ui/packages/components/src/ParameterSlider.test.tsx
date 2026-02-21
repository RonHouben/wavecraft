/**
 * ParameterSlider Component Tests
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ParameterSlider } from './ParameterSlider';

describe('ParameterSlider', () => {
  const onChange = vi.fn();

  beforeEach(() => {
    onChange.mockReset();
  });

  it('renders with parameter name as label', () => {
    render(
      <ParameterSlider
        id="gain"
        name="Gain"
        value={0.5}
        min={0}
        max={1}
        unit="%"
        onChange={onChange}
      />
    );
    expect(screen.getByText('Gain')).toBeInTheDocument();
  });

  it('displays current parameter value', () => {
    render(
      <ParameterSlider
        id="gain"
        name="Gain"
        value={0.5}
        min={0}
        max={1}
        unit="%"
        onChange={onChange}
      />
    );
    const slider = screen.getByRole('slider');
    expect(slider).toHaveValue('0.5');
  });

  it('displays formatted value with unit', () => {
    render(
      <ParameterSlider
        id="gain"
        name="Gain"
        value={0.5}
        min={0}
        max={1}
        unit="%"
        onChange={onChange}
      />
    );
    // Value is 0.5, displayed as 50.0%
    expect(screen.getByText('50.0%')).toBeInTheDocument();
  });

  it('updates value on slider change', async () => {
    render(
      <ParameterSlider
        id="gain"
        name="Gain"
        value={0.5}
        min={0}
        max={1}
        unit="%"
        onChange={onChange}
      />
    );
    const slider = screen.getByRole('slider');

    // Simulate user dragging slider to 0.8
    fireEvent.change(slider, { target: { value: '0.8' } });

    expect(onChange).toHaveBeenCalledWith(0.8);
  });

  it('respects min/max bounds', () => {
    render(
      <ParameterSlider
        id="gain"
        name="Gain"
        value={0.5}
        min={0}
        max={1}
        unit="%"
        onChange={onChange}
      />
    );
    const slider = screen.getByRole('slider');

    // Check slider has correct bounds
    expect(slider).toHaveAttribute('min', '0');
    expect(slider).toHaveAttribute('max', '1');
    expect(slider).toHaveAttribute('step', '0.001');
  });

  it('uses full frequency range and shows Hz without percent scaling', () => {
    render(
      <ParameterSlider
        id="oscillator_frequency"
        name="Oscillator Frequency"
        value={440}
        min={20}
        max={5000}
        unit="Hz"
        onChange={onChange}
      />
    );

    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('min', '20');
    expect(slider).toHaveAttribute('max', '5000');
    expect(screen.getByText('440.0 Hz')).toBeInTheDocument();
  });
});
