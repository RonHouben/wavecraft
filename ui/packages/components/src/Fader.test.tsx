import { createEvent, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Fader } from './Fader';

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

describe('Fader', () => {
  it('toggles precision visual-state metadata while Shift precision is active', () => {
    render(<Fader id="hint-fader" label="Hint" value={0.5} min={0} max={1} onChange={vi.fn()} />);

    const input = screen.getByLabelText('Hint');
    expect(input).toHaveAttribute('data-precision-active', 'false');

    fireEvent.pointerDown(input, { shiftKey: true });
    expect(input).toHaveAttribute('data-precision-active', 'true');

    fireEvent.pointerUp(input);
    expect(input).toHaveAttribute('data-precision-active', 'false');
  });

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

  it('keeps normal pointer drag behavior unchanged when Shift is not held', () => {
    const onChange = vi.fn();

    render(<Fader id="drag-fader" label="Drag" value={50} min={0} max={100} onChange={onChange} />);

    const input = screen.getByLabelText('Drag');

    fireEvent.pointerDown(input, { shiftKey: false });
    fireDragChange(input, '80', false);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(80);
  });

  it('uses finer pointer drag increments while Shift is held', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="shift-drag-fader"
        label="Shift Drag"
        value={50}
        min={0}
        max={100}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Drag');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChange(input, '80', true);
    fireDragChange(input, '90', true);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(50.833333333333336);
  });

  it('keeps precision mode active during Shift drag when change events omit Shift metadata', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="shift-metadata-omitted-fader"
        label="Shift Metadata Omitted"
        value={50}
        min={0}
        max={100}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Metadata Omitted');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChangeWithoutShiftMetadata(input, '80');
    fireDragChangeWithoutShiftMetadata(input, '90');
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(50.833333333333336);
  });

  it('exits precision mode on Shift keyup during drag even when change events omit Shift metadata', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="shift-keyup-exits-fader"
        label="Shift Keyup Exits"
        value={50}
        min={0}
        max={100}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Keyup Exits');

    fireEvent.pointerDown(input, { shiftKey: true });
    fireDragChangeWithoutShiftMetadata(input, '80');
    fireDragChangeWithoutShiftMetadata(input, '90');
    fireEvent.keyUp(window, { key: 'Shift' });
    fireDragChangeWithoutShiftMetadata(input, '100');
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenNthCalledWith(1, 50.833333333333336);
    expect(onChange).toHaveBeenNthCalledWith(2, 100);
  });

  it('adapts pointer precision mode when Shift toggles during drag', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="shift-toggle-drag-fader"
        label="Shift Toggle Drag"
        value={50}
        min={0}
        max={100}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Shift Toggle Drag');

    fireEvent.pointerDown(input, { shiftKey: false });
    fireDragChange(input, '80', false);
    fireEvent.keyDown(window, { key: 'Shift' });
    fireDragChange(input, '90', false);
    fireDragChange(input, '100', false);
    fireEvent.keyUp(window, { key: 'Shift' });
    fireDragChange(input, '70', false);
    fireEvent.pointerUp(input);

    expect(onChange).toHaveBeenCalledTimes(3);
    expect(onChange).toHaveBeenNthCalledWith(1, 80);
    expect(onChange).toHaveBeenNthCalledWith(2, 80.83333333333333);
    expect(onChange).toHaveBeenNthCalledWith(3, 70);
  });

  it('applies error semantics when state is error', () => {
    render(
      <Fader
        id="level-fader"
        label="Level"
        value={0.4}
        min={0}
        max={1}
        state="error"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Level')).toHaveAttribute('aria-invalid', 'true');
  });

  it('disables fader for disabled visual state', () => {
    render(
      <Fader
        id="gain-fader"
        label="Gain"
        value={0.4}
        min={0}
        max={1}
        state="disabled"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByLabelText('Gain')).toBeDisabled();
  });

  it('intercepts normal Arrow keys with adaptive keyboard increments', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="keyboard-fader"
        label="Keyboard"
        value={50}
        min={0}
        max={100}
        step={0.001}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Keyboard');
    const keyDownEvent = createEvent.keyDown(input, { key: 'ArrowUp', shiftKey: false });
    fireEvent(input, keyDownEvent);

    expect(keyDownEvent.defaultPrevented).toBe(true);
    expect(onChange).toHaveBeenCalledTimes(1);
    expect((onChange.mock.calls[0] as [number])[0]).toBeCloseTo(50 + 100 / 150, 10);
  });

  it('uses finer Shift+Arrow keyboard increments than normal step size', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="precision-fader"
        label="Precision"
        value={50}
        min={0}
        max={100}
        step={0.001}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Precision');

    fireEvent.keyDown(input, { key: 'ArrowUp', shiftKey: false });
    fireEvent.keyDown(input, { key: 'ArrowUp', shiftKey: true });

    expect(onChange).toHaveBeenCalledTimes(2);
    const normalValue = (onChange.mock.calls[0] as [number])[0];
    const precisionValue = (onChange.mock.calls[1] as [number])[0];
    const normalDelta = Math.abs(normalValue - 50);
    const precisionDelta = Math.abs(precisionValue - 50);

    expect(normalDelta).toBeCloseTo(100 / 150, 10);
    expect(precisionDelta).toBeCloseTo(100 / 150 / 12, 10);
    expect(normalDelta).toBeGreaterThan(precisionDelta * 10);
  });

  it('uses Shift precision mode when getModifierState reports Shift and shiftKey is false', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="precision-modifier-fallback-fader"
        label="Precision Modifier Fallback"
        value={50}
        min={0}
        max={100}
        step={0.001}
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

    expect(keyDownEvent.defaultPrevented).toBe(true);
    expect(onChange).toHaveBeenCalledTimes(1);
    expect((onChange.mock.calls[0] as [number])[0]).toBeCloseTo(50 + 100 / 150 / 12, 10);
  });
});
