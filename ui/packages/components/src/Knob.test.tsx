import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Knob } from './Knob';

describe('Knob', () => {
  it('renders with accessible label and formatted value', () => {
    render(
      <Knob id="gain-knob" label="Gain" value={0.5} min={0} max={1} unit="%" onChange={vi.fn()} />
    );

    expect(screen.getByLabelText('Gain')).toBeInTheDocument();
    expect(screen.getByText('50.0%')).toBeInTheDocument();
  });

  it('calls onChange when value changes', () => {
    const onChange = vi.fn();

    render(
      <Knob id="freq-knob" label="Freq" value={440} min={20} max={20000} onChange={onChange} />
    );

    const input = screen.getByLabelText('Freq');
    fireEvent.change(input, { target: { value: '880' } });

    expect(onChange).toHaveBeenCalledWith(880);
  });
});
