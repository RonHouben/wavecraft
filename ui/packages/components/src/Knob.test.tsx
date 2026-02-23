import { createEvent, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Knob } from './Knob';

function fireDragChange(input: HTMLElement, value: string, shiftKey: boolean = false): void {
  const changeEvent = createEvent.change(input, { target: { value }, shiftKey });
  Object.defineProperty(changeEvent, 'getModifierState', {
    configurable: true,
    value: (keyArg: string): boolean => keyArg === 'Shift' && shiftKey,
  });
  fireEvent(input, changeEvent);
}

function fireDragChangeWithoutShiftMetadata(input: HTMLElement, value: string): void {
  const changeEvent = createEvent.change(input, { target: { value } });
  fireEvent(input, changeEvent);
}

describe('Knob', () => {
  it('toggles precision visual-state metadata while Shift precision is active', () => {
    render(<Knob id="hint-knob" label="Hint" value={0.5} min={0} max={1} onChange={vi.fn()} />);

    const input = screen.getByLabelText('Hint');
    expect(input).toHaveAttribute('data-precision-active', 'false');

    fireEvent.pointerDown(input, { shiftKey: true });
    expect(input).toHaveAttribute('data-precision-active', 'true');

    fireEvent.pointerUp(input);
    expect(input).toHaveAttribute('data-precision-active', 'false');
  });

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

  it('keeps normal pointer drag behavior unchanged when Shift is not held', () => {
    const onChange = vi.fn();

    render(<Knob id="drag-knob" label="Drag" value={0.5} min={0} max={1} onChange={onChange} />);

    const input = screen.getByLabelText('Drag');

    fireEvent.pointerDown(input, { shiftKey: false });
    fireDragChange(input, '0.8', false);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(0.8);
  });

  it('uses finer pointer drag increments while Shift is held', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="shift-drag-knob"
        label="Shift Drag"
        value={0.5}
        min={0}
        max={1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Drag');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChange(input, '0.8', true);
    fireDragChange(input, '0.9', true);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(0.5083333333333333);
  });

  it('keeps precision mode active during Shift drag when change events omit Shift metadata', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="shift-metadata-omitted-knob"
        label="Shift Metadata Omitted"
        value={0.5}
        min={0}
        max={1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Metadata Omitted');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChangeWithoutShiftMetadata(input, '0.8');
    fireDragChangeWithoutShiftMetadata(input, '0.9');
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(0.5083333333333333);
  });

  it('exits precision mode on Shift keyup during drag even when change events omit Shift metadata', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="shift-keyup-exits-knob"
        label="Shift Keyup Exits"
        value={0.5}
        min={0}
        max={1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Keyup Exits');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChangeWithoutShiftMetadata(input, '0.8');
    fireDragChangeWithoutShiftMetadata(input, '0.9');
    fireEvent.keyUp(window, { key: 'Shift' });
    fireDragChangeWithoutShiftMetadata(input, '1.0');
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenNthCalledWith(1, 0.5083333333333333);
    expect(onChange).toHaveBeenNthCalledWith(2, 1);
  });

  it('adapts pointer precision mode when Shift toggles during drag', () => {
    const onChange = vi.fn();

    render(
      <Knob
        id="shift-toggle-drag-knob"
        label="Shift Toggle Drag"
        value={0.5}
        min={0}
        max={1}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Toggle Drag');

    fireEvent.pointerDown(input, { shiftKey: false });
    fireDragChange(input, '0.8', false);
    fireEvent.keyDown(window, { key: 'Shift' });
    fireDragChange(input, '0.9', false);
    fireDragChange(input, '1.0', false);
    fireEvent.keyUp(window, { key: 'Shift' });
    fireDragChange(input, '0.7', false);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(3);
    expect(onChange).toHaveBeenNthCalledWith(1, 0.8);
    expect(onChange).toHaveBeenNthCalledWith(2, 0.8083333333333333);
    expect(onChange).toHaveBeenNthCalledWith(3, 0.7);
  });

  it('turns precision cue on when Shift is pressed during an active drag', () => {
    render(
      <Knob
        id="drag-shift-press-knob"
        label="Drag Shift Press"
        value={0.5}
        min={0}
        max={1}
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Drag Shift Press');

    fireEvent.pointerDown(input, { shiftKey: false });
    expect(input).toHaveAttribute('data-precision-active', 'false');

    fireEvent.keyDown(window, { key: 'Shift' });
    expect(input).toHaveAttribute('data-precision-active', 'true');

    fireEvent.pointerUp(input);
  });

  it('turns precision cue off immediately when Shift is released during an active drag', () => {
    render(
      <Knob
        id="drag-shift-release-knob"
        label="Drag Shift Release"
        value={0.5}
        min={0}
        max={1}
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Drag Shift Release');

    fireEvent.pointerDown(input, { shiftKey: true });
    expect(input).toHaveAttribute('data-precision-active', 'true');

    fireEvent.keyUp(window, { key: 'Shift' });
    expect(input).toHaveAttribute('data-precision-active', 'false');

    fireEvent.pointerUp(input);
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
    expect(onChange).toHaveBeenNthCalledWith(2, 0.5083333333333333);
    expect(onChange).toHaveBeenNthCalledWith(3, 0.49166666666666664);

    const normalDelta = Math.abs((onChange.mock.calls[0] as [number])[0] - 0.5);
    const precisionDelta = Math.abs((onChange.mock.calls[1] as [number])[0] - 0.5);
    expect(normalDelta).toBeGreaterThan(precisionDelta * 10);
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
    expect(onChange).toHaveBeenCalledWith(0.5083333333333333);
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
    expect(precisionArrowUpValue).toBeCloseTo(1011.1, 6);
    expect(precisionArrowDownValue).toBeCloseTo(988.9, 6);

    const normalDelta = Math.abs(normalArrowUpValue - 1000);
    const precisionDelta = Math.abs(precisionArrowUpValue - 1000);
    expect(normalDelta).toBeGreaterThan(precisionDelta * 10);
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

  it('reserves a stable value-label width for wide unit ranges', () => {
    render(
      <Knob
        id="freq-layout-knob"
        label="Frequency Layout"
        value={20}
        min={20}
        max={20000}
        unit="Hz"
        onChange={vi.fn()}
      />
    );

    const valueLabel = screen.getByText('20.00 Hz');
    expect(valueLabel).toHaveStyle({ width: '11ch' });
  });

  it('uses min-width sizing and avoids value-row clipping in small layout', () => {
    render(
      <Knob
        id="freq-layout-small-knob"
        label="Frequency Layout Small"
        value={20}
        min={20}
        max={20000}
        unit="Hz"
        size="sm"
        onChange={vi.fn()}
      />
    );

    const input = screen.getByRole('slider', { name: 'Frequency Layout Small' });
    const root = input.closest('.group');
    expect(root).not.toBeNull();
    expect(root).toHaveClass('min-w-[88px]');
    expect(root).not.toHaveClass('w-[72px]');

    const valueLabel = screen.getByText('20.00 Hz');
    const valueRow = valueLabel.parentElement;
    expect(valueLabel).toHaveStyle({ width: '11ch' });
    expect(valueRow).not.toBeNull();
    expect(valueRow).not.toHaveClass('overflow-hidden');
  });
});
