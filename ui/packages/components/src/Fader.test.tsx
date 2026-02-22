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

function mockSliderRect(input: HTMLElement, top: number, height: number) {
  return vi.spyOn(input, 'getBoundingClientRect').mockReturnValue({
    x: 0,
    y: top,
    top,
    left: 0,
    right: 20,
    bottom: top + height,
    width: 20,
    height,
    toJSON: (): Record<string, never> => ({}),
  } as DOMRect);
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

  it('keeps percent readout width stable with a fixed numeric slot', () => {
    const { rerender } = render(
      <Fader
        id="mix-stable-width"
        label="Mix Stable Width"
        value={0}
        min={0}
        max={1}
        unit="%"
        onChange={vi.fn()}
      />
    );

    const initialValue = screen.getByText('0.0%');
    expect(initialValue).toHaveClass('min-w-[6ch]');
    expect(initialValue).toHaveClass('text-right');

    rerender(
      <Fader
        id="mix-stable-width"
        label="Mix Stable Width"
        value={1}
        min={0}
        max={1}
        unit="%"
        onChange={vi.fn()}
      />
    );

    const maxValue = screen.getByText('100.0%');
    expect(maxValue).toHaveClass('min-w-[6ch]');
    expect(maxValue).toHaveClass('text-right');
  });

  it('defaults to horizontal orientation', () => {
    render(<Fader id="pan-fader" label="Pan" value={0} min={-1} max={1} onChange={vi.fn()} />);

    const input = screen.getByLabelText('Pan');
    const controlContainer = input.parentElement;

    expect(controlContainer).not.toBeNull();
    expect(controlContainer).toHaveClass('w-full');
    expect(controlContainer).toHaveClass('h-9');
    expect(input).toHaveClass('w-full');
    expect(input).toHaveClass('h-2');
    expect(input).toHaveClass('bg-transparent');
    expect(input).toHaveClass('accent-accent');
    expect(input).not.toHaveClass('-rotate-90');

    const rail = controlContainer?.querySelector('[data-slot="horizontal-rail"]');
    const fill = controlContainer?.querySelector('[data-slot="horizontal-fill"]');

    expect(rail).not.toBeNull();
    expect(rail).toHaveClass('h-2');
    expect(rail).toHaveClass('bg-plugin-border');
    expect(fill).not.toBeNull();
    expect(fill).toHaveClass('h-1');
    expect(fill).toHaveClass('bg-accent');
  });

  it('supports vertical orientation', () => {
    render(
      <Fader
        id="level-fader-vertical"
        label="Level Vertical"
        value={0}
        min={-1}
        max={1}
        orientation="vertical"
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Level Vertical');
    const controlContainer = input.parentElement;

    expect(controlContainer).not.toBeNull();
    expect(controlContainer).toHaveClass('h-[160px]');
    expect(controlContainer).toHaveClass('w-10');
    expect(input).toHaveClass('h-full');
    expect(input).toHaveClass('w-2');
    expect(input).toHaveClass('[direction:rtl]');
    expect(input).toHaveClass('[writing-mode:vertical-lr]');
    expect(input).toHaveClass('[appearance:slider-vertical]');
    expect(input).toHaveClass('[-webkit-appearance:slider-vertical]');
    expect(input).toHaveClass('rounded-full');
    expect(input).toHaveClass('bg-plugin-border');
    expect(input).toHaveClass('accent-accent');
    expect(input).not.toHaveClass('w-full');
    expect(input).not.toHaveClass('-rotate-90');
  });

  it('uses shared frame and rail treatment for both orientations while keeping orientation-specific geometry', () => {
    const { rerender } = render(
      <Fader id="family-fader" label="Family" value={0.5} min={0} max={1} onChange={vi.fn()} />
    );

    const horizontalInput = screen.getByLabelText('Family');
    const horizontalContainer = horizontalInput.parentElement;

    expect(horizontalContainer).not.toBeNull();
    expect(horizontalContainer).toHaveClass(
      'rounded-md',
      'border',
      'border-plugin-border',
      'bg-plugin-dark',
      'p-2',
      'w-full'
    );
    expect(horizontalInput).toHaveClass(
      'rounded-full',
      'h-2',
      'w-full',
      'bg-transparent',
      'accent-accent'
    );

    rerender(
      <Fader
        id="family-fader"
        label="Family"
        value={0.5}
        min={0}
        max={1}
        orientation="vertical"
        onChange={vi.fn()}
      />
    );

    const verticalInput = screen.getByLabelText('Family');
    const verticalContainer = verticalInput.parentElement;

    expect(verticalContainer).not.toBeNull();
    expect(verticalContainer).toHaveClass(
      'rounded-md',
      'border',
      'border-plugin-border',
      'bg-plugin-dark',
      'p-2'
    );
    expect(verticalInput).toHaveClass(
      'rounded-full',
      'h-full',
      'w-2',
      'bg-plugin-border',
      'accent-accent'
    );
    expect(verticalInput).toHaveClass('[writing-mode:vertical-lr]');
  });

  it('renders horizontal blue fill width from the normalized value', () => {
    render(<Fader id="fill-fader" label="Fill" value={0.25} min={0} max={1} onChange={vi.fn()} />);

    const input = screen.getByLabelText('Fill');
    const controlContainer = input.parentElement;
    const fill = controlContainer?.querySelector('[data-slot="horizontal-fill"]');

    expect(fill).not.toBeNull();
    expect(fill).toHaveAttribute('style', 'width: calc(25.000% + 4.500px);');
  });

  it('uses full-width horizontal geometry across size variants while preserving height footprints', () => {
    const { rerender } = render(
      <Fader
        id="size-fader"
        label="Size"
        value={0.5}
        min={0}
        max={1}
        size="sm"
        onChange={vi.fn()}
      />
    );

    const input = screen.getByLabelText('Size');
    const smContainer = input.parentElement;

    expect(smContainer).not.toBeNull();
    expect(smContainer).toHaveClass('w-full');
    expect(smContainer).toHaveClass('h-8');

    rerender(
      <Fader
        id="size-fader"
        label="Size"
        value={0.5}
        min={0}
        max={1}
        size="lg"
        onChange={vi.fn()}
      />
    );

    const lgContainer = screen.getByLabelText('Size').parentElement;

    expect(lgContainer).not.toBeNull();
    expect(lgContainer).toHaveClass('w-full');
    expect(lgContainer).toHaveClass('h-10');
  });

  it('keeps horizontal rail/fill behind the slider thumb for proper layering', () => {
    render(<Fader id="layer-fader" label="Layer" value={0.6} min={0} max={1} onChange={vi.fn()} />);

    const input = screen.getByLabelText('Layer');
    const controlContainer = input.parentElement;
    const railWrapper = controlContainer?.querySelector(
      '[data-slot="horizontal-rail"]'
    )?.parentElement;

    expect(input).toHaveClass('relative');
    expect(input).toHaveClass('z-10');
    expect(railWrapper).not.toBeNull();
    expect(railWrapper).toHaveClass('z-0');
  });

  it('uses the same interaction styling language for horizontal and vertical thumbs', () => {
    const { rerender } = render(
      <Fader id="parity-fader" label="Parity" value={0.5} min={0} max={1} onChange={vi.fn()} />
    );

    const sharedInteractionClasses = [
      'slider-thumb',
      'appearance-none',
      'rounded-full',
      'accent-accent',
      'ring-1',
      'ring-inset',
      'ring-plugin-dark/80',
    ];

    const horizontalInput = screen.getByLabelText('Parity');
    for (const className of sharedInteractionClasses) {
      expect(horizontalInput).toHaveClass(className);
    }

    rerender(
      <Fader
        id="parity-fader"
        label="Parity"
        value={0.5}
        min={0}
        max={1}
        orientation="vertical"
        onChange={vi.fn()}
      />
    );

    const verticalInput = screen.getByLabelText('Parity');
    for (const className of sharedInteractionClasses) {
      expect(verticalInput).toHaveClass(className);
    }
  });

  it('maps vertical pointer dragging by geometry so upward movement increases output value', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="vertical-drag-fader"
        label="Vertical Drag"
        value={0.4}
        min={0}
        max={1}
        orientation="vertical"
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Vertical Drag');

    const rectSpy = mockSliderRect(input, 100, 160);

    fireEvent.pointerDown(input, { pointerId: 1, clientY: 180, shiftKey: false });
    fireEvent.pointerMove(input, { pointerId: 1, clientY: 100, shiftKey: false });
    fireEvent.pointerUp(input, { pointerId: 1, clientY: 100 });

    rectSpy.mockRestore();

    expect(onChange).toHaveBeenCalled();
    const calls = onChange.mock.calls as [number][];
    expect(calls[calls.length - 1]?.[0]).toBeCloseTo(1, 10);
  });

  it('maps vertical pointer dragging by geometry so downward movement decreases output value', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="vertical-drag-down-fader"
        label="Vertical Drag Down"
        value={0.6}
        min={0}
        max={1}
        orientation="vertical"
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Vertical Drag Down');

    const rectSpy = mockSliderRect(input, 100, 160);

    fireEvent.pointerDown(input, { pointerId: 2, clientY: 120, shiftKey: false });
    fireEvent.pointerMove(input, { pointerId: 2, clientY: 260, shiftKey: false });
    fireEvent.pointerUp(input, { pointerId: 2, clientY: 260 });

    rectSpy.mockRestore();

    expect(onChange).toHaveBeenCalled();
    const calls = onChange.mock.calls as [number][];
    expect(calls[calls.length - 1]?.[0]).toBeCloseTo(0, 10);
  });

  it('applies slider orientation semantics for assistive technologies', () => {
    const { rerender } = render(
      <Fader id="a11y-fader" label="A11y" value={0.5} min={0} max={1} onChange={vi.fn()} />
    );

    expect(screen.getByRole('slider', { name: 'A11y' })).toHaveAttribute(
      'aria-orientation',
      'horizontal'
    );

    rerender(
      <Fader
        id="a11y-fader"
        label="A11y"
        value={0.5}
        min={0}
        max={1}
        orientation="vertical"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByRole('slider', { name: 'A11y' })).toHaveAttribute(
      'aria-orientation',
      'vertical'
    );
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

  it('keeps keyboard arrow behavior intact in vertical mode', () => {
    const onChange = vi.fn();

    render(
      <Fader
        id="vertical-keyboard-fader"
        label="Vertical Keyboard"
        value={0.5}
        min={0}
        max={1}
        orientation="vertical"
        step={0.001}
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText('Vertical Keyboard');

    fireEvent.keyDown(input, { key: 'ArrowUp', shiftKey: false });
    fireEvent.keyDown(input, { key: 'ArrowDown', shiftKey: false });

    expect(onChange).toHaveBeenCalledTimes(2);
    expect((onChange.mock.calls[0] as [number])[0]).toBeCloseTo(0.5 + 1 / 150, 10);
    expect((onChange.mock.calls[1] as [number])[0]).toBeCloseTo(0.5 - 1 / 150, 10);
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
