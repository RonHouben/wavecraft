import { createEvent, fireEvent, render, screen } from '@testing-library/react';
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

  it('disables knob and sets busy semantics in loading state', () => {
    render(
      <Knob
        id="gain-knob"
        label="Gain"
        value={0.5}
        min={0}
        max={1}
        state="loading"
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Gain');
    expect(input).toBeDisabled();
    expect(input).toHaveAttribute('aria-busy', 'true');
  });

  it('disables knob for disabled visual state', () => {
    render(
      <Knob
        id="mix-knob"
        label="Mix"
        value={0.5}
        min={0}
        max={1}
        state="disabled"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Mix')).toBeDisabled();
  });

  it('supports slider keyboard controls with fine, coarse, and edge jumps', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="keyboard-knob"
        label="Keyboard"
        value={0.5}
        min={0}
        max={1}
        step={0.1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Keyboard');

    fireEvent.keyDown(input, { key: 'ArrowUp' });
    fireEvent.keyDown(input, { key: 'ArrowLeft' });
    fireEvent.keyDown(input, { key: 'PageUp' });
    fireEvent.keyDown(input, { key: 'PageDown' });
    fireEvent.keyDown(input, { key: 'Home' });
    fireEvent.keyDown(input, { key: 'End' });

    expect(onChange).toHaveBeenNthCalledWith(1, 0.6);
    expect(onChange).toHaveBeenNthCalledWith(2, 0.4);
    expect(onChange).toHaveBeenNthCalledWith(3, 1);
    expect(onChange).toHaveBeenNthCalledWith(4, 0);
    expect(onChange).toHaveBeenNthCalledWith(5, 0);
    expect(onChange).toHaveBeenNthCalledWith(6, 1);
  });

  it('uses precision mode for Shift+Arrow with smaller deltas than normal arrows', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="precision-knob"
        label="Precision"
        value={0.5}
        min={0}
        max={1}
        step={0.1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Precision');

    fireEvent.keyDown(input, { key: 'ArrowUp' });
    fireEvent.keyDown(input, { key: 'ArrowUp', shiftKey: true });
    fireEvent.keyDown(input, { key: 'ArrowDown', shiftKey: true });

    expect(onChange).toHaveBeenCalledTimes(3);
    expect(onChange).toHaveBeenNthCalledWith(1, 0.6);
    expect(onChange).toHaveBeenNthCalledWith(2, 0.525);
    expect(onChange).toHaveBeenNthCalledWith(3, 0.475);
  });

  it('uses precision mode when getModifierState reports Shift even if shiftKey is false', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="precision-modifier-fallback-knob"
        label="Precision Modifier Fallback"
        value={0.5}
        min={0}
        max={1}
        step={0.1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Precision Modifier Fallback');
    const keyDownEvent = createEvent.keyDown(input, { key: 'ArrowUp', shiftKey: false });
    Object.defineProperty(keyDownEvent, 'getModifierState', {
      configurable: true,
      value: (keyArg: string): boolean => keyArg === 'Shift',
    });

    fireEvent(input, keyDownEvent);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(0.525);
  });

  it('uses adaptive keyboard steps for wide ranges while keeping page keys coarser', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="wide-range-knob"
        label="Wide"
        value={1000}
        min={20}
        max={20000}
        step={0.001}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Wide');

    fireEvent.keyDown(input, { key: 'ArrowUp' });
    fireEvent.keyDown(input, { key: 'PageUp' });
    fireEvent.keyDown(input, { key: 'PageDown' });
    fireEvent.keyDown(input, { key: 'End' });
    fireEvent.keyDown(input, { key: 'Home' });

    expect(onChange).toHaveBeenCalledTimes(5);

    const [arrowUpValue] = onChange.mock.calls[0] as [number];
    const [pageUpValue] = onChange.mock.calls[1] as [number];
    const [pageDownValue] = onChange.mock.calls[2] as [number];

    expect(arrowUpValue).toBeCloseTo(1133.2, 6);
    expect(pageUpValue).toBeCloseTo(2065.6, 6);
    expect(pageDownValue).toBe(20);
    expect(onChange).toHaveBeenNthCalledWith(4, 20000);
    expect(onChange).toHaveBeenNthCalledWith(5, 20);
  });

  it('applies adaptive precision mode for Shift+Arrow on wide ranges', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="wide-precision-knob"
        label="Wide Precision"
        value={1000}
        min={20}
        max={20000}
        step={0.001}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Wide Precision');

    fireEvent.keyDown(input, { key: 'ArrowUp' });
    fireEvent.keyDown(input, { key: 'ArrowUp', shiftKey: true });
    fireEvent.keyDown(input, { key: 'ArrowDown', shiftKey: true });

    expect(onChange).toHaveBeenCalledTimes(3);

    const [normalArrowUpValue] = onChange.mock.calls[0] as [number];
    const [precisionArrowUpValue] = onChange.mock.calls[1] as [number];
    const [precisionArrowDownValue] = onChange.mock.calls[2] as [number];

    expect(normalArrowUpValue).toBeCloseTo(1133.2, 6);
    expect(precisionArrowUpValue).toBeCloseTo(1033.3, 6);
    expect(precisionArrowDownValue).toBeCloseTo(966.7, 6);
  });

  it('exposes aria-valuetext with formatted unit', () => {
    render(
      <Knob
        id="freq-knob"
        label="Frequency"
        value={9756.43}
        min={20}
        max={20000}
        unit="Hz"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByRole('slider', { name: 'Frequency' })).toHaveAttribute(
      'aria-valuetext',
      '9756.43 Hz'
    );
  });
});
