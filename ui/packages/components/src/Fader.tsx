import { useEffect, useRef } from 'react';
import type { ControlVisualState, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

export interface FaderProps {
  readonly id: string;
  readonly label: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly onChange: (value: number) => void;
  readonly disabled?: boolean;
  readonly orientation?: 'horizontal' | 'vertical';
  readonly pluginState?: PluginVisualState;
  readonly size?: 'sm' | 'md' | 'lg';
  readonly state?: ControlVisualState;
  readonly step?: number;
  readonly unit?: string;
}

const horizontalLengthClassMap: Record<NonNullable<FaderProps['size']>, string> = {
  sm: 'w-[120px]',
  md: 'w-[180px]',
  lg: 'w-[240px]',
};

const verticalLengthClassMap: Record<NonNullable<FaderProps['size']>, string> = {
  sm: 'h-[120px]',
  md: 'h-[160px]',
  lg: 'h-[220px]',
};

const SHIFT_PRECISION_HINT = 'Hold Shift for fine adjust';
const SHIFT_DRAG_PRECISION_DIVISOR = 12;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function isShiftPrecisionActive(event: {
  readonly shiftKey: boolean;
  readonly getModifierState?: (keyArg: 'Shift') => boolean;
}): boolean {
  if (event.shiftKey) {
    return true;
  }

  return event.getModifierState?.('Shift') ?? false;
}

function resolveShiftFromChangeEvent(event: {
  readonly shiftKey?: boolean;
  readonly getModifierState?: (keyArg: 'Shift') => boolean;
}): boolean {
  if (event.shiftKey === true) {
    return true;
  }

  return event.getModifierState?.('Shift') ?? false;
}

function getKeyboardSteps(
  min: number,
  max: number,
  step: number
): {
  readonly arrowStep: number;
  readonly precisionArrowStep: number;
} {
  const range = Math.max(0, max - min);
  const safeStep = Number.isFinite(step) && step > 0 ? step : 0.001;
  const arrowStep = Math.max(safeStep, range / 150);

  return {
    arrowStep,
    precisionArrowStep: arrowStep / 12,
  };
}

function formatValue(value: number, unit?: string): string {
  if (!unit) {
    return value.toFixed(3);
  }

  if (unit === '%') {
    return `${(value * 100).toFixed(1)}%`;
  }

  return `${value.toFixed(2)} ${unit}`;
}

export function Fader({
  disabled = false,
  id,
  label,
  max,
  min,
  onChange,
  orientation = 'vertical',
  pluginState,
  size = 'md',
  state = 'default',
  step = 0.001,
  unit,
  value,
}: Readonly<FaderProps>): React.JSX.Element {
  const isPointerDragActiveRef = useRef(false);
  const isShiftPressedDuringDragRef = useRef(false);
  const shiftDragAnchorRawValueRef = useRef<number | null>(null);
  const shiftDragAnchorOutputValueRef = useRef<number | null>(null);
  const latestOutputValueRef = useRef(value);

  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isVertical = orientation === 'vertical';
  const keyboardSteps = getKeyboardSteps(min, max, step);
  const clampedValue = clamp(value, min, max);

  function resetShiftDragAnchors(): void {
    shiftDragAnchorRawValueRef.current = null;
    shiftDragAnchorOutputValueRef.current = null;
  }

  useEffect(() => {
    latestOutputValueRef.current = clampedValue;
  }, [clampedValue]);

  useEffect(() => {
    if (!isPointerDragActiveRef.current) {
      return;
    }

    function handleWindowKeyDown(event: KeyboardEvent): void {
      if (event.key === 'Shift') {
        isShiftPressedDuringDragRef.current = true;
      }
    }

    function handleWindowKeyUp(event: KeyboardEvent): void {
      if (event.key === 'Shift') {
        isShiftPressedDuringDragRef.current = false;
        resetShiftDragAnchors();
      }
    }

    window.addEventListener('keydown', handleWindowKeyDown);
    window.addEventListener('keyup', handleWindowKeyUp);

    return () => {
      window.removeEventListener('keydown', handleWindowKeyDown);
      window.removeEventListener('keyup', handleWindowKeyUp);
    };
  });

  return (
    <div className="group inline-flex flex-col items-center gap-2">
      <label
        htmlFor={id}
        className="text-type-xs uppercase tracking-wide text-plugin-text-secondary"
      >
        {label}
      </label>

      <div
        className={mergeClassNames(
          'inline-flex items-center justify-center rounded-md border border-plugin-border bg-plugin-dark p-2',
          isVertical ? verticalLengthClassMap[size] : horizontalLengthClassMap[size],
          getControlStateClass({ disabled: isDisabled, pluginState, state }),
          isError ? 'border-meter-clip' : ''
        )}
      >
        <input
          id={id}
          type="range"
          min={min}
          max={max}
          step={step}
          value={clampedValue}
          disabled={isDisabled}
          onPointerDown={(event): void => {
            isPointerDragActiveRef.current = true;
            isShiftPressedDuringDragRef.current = event.shiftKey;
            resetShiftDragAnchors();
          }}
          onPointerUp={(): void => {
            isPointerDragActiveRef.current = false;
            isShiftPressedDuringDragRef.current = false;
            resetShiftDragAnchors();
          }}
          onPointerCancel={(): void => {
            isPointerDragActiveRef.current = false;
            isShiftPressedDuringDragRef.current = false;
            resetShiftDragAnchors();
          }}
          onKeyDown={(event): void => {
            if (isDisabled) {
              return;
            }

            const isPrecisionMode = isShiftPrecisionActive(event);

            if (event.key === 'ArrowUp' || event.key === 'ArrowRight') {
              event.preventDefault();
              const delta = isPrecisionMode
                ? keyboardSteps.precisionArrowStep
                : keyboardSteps.arrowStep;
              const nextValue = clamp(clampedValue + delta, min, max);
              if (nextValue !== clampedValue) {
                onChange(nextValue);
              }
              return;
            }

            if (event.key === 'ArrowDown' || event.key === 'ArrowLeft') {
              event.preventDefault();
              const delta = isPrecisionMode
                ? -keyboardSteps.precisionArrowStep
                : -keyboardSteps.arrowStep;
              const nextValue = clamp(clampedValue + delta, min, max);
              if (nextValue !== clampedValue) {
                onChange(nextValue);
              }
            }
          }}
          onChange={(event): void => {
            const rawValue = Number.parseFloat(event.currentTarget.value);
            const nativeEvent = event.nativeEvent as {
              readonly shiftKey?: boolean;
              readonly getModifierState?: (keyArg: 'Shift') => boolean;
            };
            const isShiftActiveOnEvent = resolveShiftFromChangeEvent(nativeEvent);

            if (isPointerDragActiveRef.current) {
              isShiftPressedDuringDragRef.current = isShiftActiveOnEvent;
            }

            const isShiftPrecisionMode = isPointerDragActiveRef.current
              ? isShiftPressedDuringDragRef.current
              : isShiftActiveOnEvent;

            if (!isShiftPrecisionMode) {
              resetShiftDragAnchors();
              latestOutputValueRef.current = rawValue;
              onChange(rawValue);
              return;
            }

            if (shiftDragAnchorRawValueRef.current === null) {
              shiftDragAnchorRawValueRef.current = rawValue;
              shiftDragAnchorOutputValueRef.current = latestOutputValueRef.current;
              return;
            }

            const anchorRawValue = shiftDragAnchorRawValueRef.current;
            const anchorOutputValue =
              shiftDragAnchorOutputValueRef.current ?? latestOutputValueRef.current;
            const precisionValue = clamp(
              anchorOutputValue + (rawValue - anchorRawValue) / SHIFT_DRAG_PRECISION_DIVISOR,
              min,
              max
            );

            latestOutputValueRef.current = precisionValue;
            onChange(precisionValue);
          }}
          aria-busy={isLoading || undefined}
          aria-invalid={isError || undefined}
          data-state={state}
          data-plugin-state={pluginState}
          className={mergeClassNames(
            'slider-thumb h-2 appearance-none rounded-sm bg-plugin-border',
            focusRingClass,
            isVertical ? 'w-full -rotate-90' : 'w-full'
          )}
        />
      </div>

      <div className="relative inline-flex items-center gap-1">
        <span className="font-mono text-type-sm tabular-nums text-plugin-text-primary">
          {formatValue(clampedValue, unit)}
        </span>
        {badgeLabel ? (
          <span
            className={mergeClassNames(
              'rounded-sm border px-1 py-0.5 font-mono text-[10px] leading-none',
              getStateBadgeClass(pluginState)
            )}
            aria-hidden="true"
          >
            {badgeLabel}
          </span>
        ) : null}
        <span
          aria-hidden="true"
          className="pointer-events-none absolute left-1/2 top-full mt-1 -translate-x-1/2 whitespace-nowrap text-type-xs text-plugin-text-secondary opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 motion-reduce:transition-none"
        >
          {SHIFT_PRECISION_HINT}
        </span>
      </div>
    </div>
  );
}
