import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ParameterSlider } from './ParameterSlider';

describe('ParameterSlider reconnect recovery', () => {
  it('updates rendered value when parent rerenders with latest data', () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <ParameterSlider
        id="test_tone_frequency"
        name="Test Tone Frequency"
        value={440}
        min={20}
        max={20000}
        unit="Hz"
        onChange={onChange}
      />
    );

    rerender(
      <ParameterSlider
        id="test_tone_frequency"
        name="Test Tone Frequency"
        value={880}
        min={20}
        max={20000}
        unit="Hz"
        onChange={onChange}
      />
    );

    expect(screen.getByText('Test Tone Frequency')).toBeInTheDocument();
    expect(screen.getByText('880.0 Hz')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toBeInTheDocument();
  });
});
