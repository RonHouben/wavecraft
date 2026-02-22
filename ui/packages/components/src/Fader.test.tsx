import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Fader } from './Fader';

describe('Fader', () => {
  it('renders label and formatted value', () => {
    render(
      <Fader id="mix-fader" label="Mix" value={0.25} min={0} max={1} unit="%" onChange={vi.fn()} />
    );

    expect(screen.getByLabelText('Mix')).toBeInTheDocument();
    expect(screen.getByText('25.0%')).toBeInTheDocument();
  });

  it('supports horizontal orientation', () => {
    render(
      <Fader
        id="pan-fader"
        label="Pan"
        value={0}
        min={-1}
        max={1}
        orientation="horizontal"
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Pan');
    expect(input).toHaveClass('w-full');
    expect(input).not.toHaveClass('-rotate-90');
  });

  it('calls onChange when fader value changes', () => {
    const onChange = vi.fn();

    render(
      <Fader id="level-fader" label="Level" value={0.4} min={0} max={1} onChange={onChange} />
    );

    const input = screen.getByLabelText('Level');
    fireEvent.change(input, { target: { value: '0.8' } });

    expect(onChange).toHaveBeenCalledWith(0.8);
  });
});
