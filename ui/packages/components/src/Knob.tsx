import { useCallback, useEffect, useRef } from 'react';
import type { ControlVisualState, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

export interface KnobProps {
  readonly id: string;
  readonly label: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly onChange: (value: number) => void;
  readonly step?: number;
  readonly disabled?: boolean;
  readonly pluginState?: PluginVisualState;
  readonly size?: 'sm' | 'md' | 'lg';
  readonly state?: ControlVisualState;
  readonly unit?: string;
}

const knobSizeClassMap: Record<NonNullable<KnobProps['size']>, string> = {
  sm: 'h-8 w-8',
  md: 'h-11 w-11',
  lg: 'h-14 w-14',
};

const knobWidthClassMap: Record<NonNullable<KnobProps['size']>, string> = {
  sm: 'w-[72px]',
  md: 'w-[88px]',
  lg: 'w-[88px]',
};

const KNOB_SWEEP_START_DEG = -135;
const KNOB_SWEEP_RANGE_DEG = 270;
const SHIFT_PRECISION_HINT = 'Hold Shift for fine adjust';
const SHIFT_DRAG_PRECISION_DIVISOR = 12;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function getKeyboardSteps(
  min: number,
  max: number,
  step: number
): {
  readonly arrowStep: number;
  readonly precisionArrowStep: number;
  readonly pageStep: number;
} {
  const range = Math.max(0, max - min);
  const safeStep = Number.isFinite(step) && step > 0 ? step : 0.001;
  const arrowStep = Math.max(safeStep, range / 150);
  const precisionArrowStep = arrowStep / 12;

  return {
    arrowStep,
    precisionArrowStep,
    pageStep: arrowStep * 8,
  };
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

function formatValue(value: number, unit?: string): string {
  if (!unit) {
    return value.toFixed(3);
  }

  if (unit === '%') {
    return `${(value * 100).toFixed(1)}%`;
  }

  return `${value.toFixed(2)} ${unit}`;
}

export function Knob({
  disabled = false,
  id,
  label,
  max,
  min,
  onChange,
  step = 0.001,
  pluginState,
  size = 'md',
  state = 'default',
  unit,
  value,
}: Readonly<KnobProps>): React.JSX.Element {
  const isPointerDragActiveRef = useRef(false);
  const isShiftPressedDuringDragRef = useRef(false);
  const shiftDragAnchorRawValueRef = useRef<number | null>(null);
  const shiftDragAnchorOutputValueRef = useRef<number | null>(null);
  const latestOutputValueRef = useRef(value);

  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const clampedValue = clamp(value, min, max);
  const normalized = (clampedValue - min) / (max - min || 1);
  const angle = KNOB_SWEEP_START_DEG + normalized * KNOB_SWEEP_RANGE_DEG;
  const badgeLabel = getStateBadgeLabel(pluginState);
  const formattedValue = formatValue(clampedValue, unit);
  const keyboardSteps = getKeyboardSteps(min, max, step);

  function applyKeyboardDelta(delta: number): void {
    const nextValue = clamp(clampedValue + delta, min, max);
    if (nextValue !== clampedValue) {
      onChange(nextValue);
    }
  }

  const resetShiftDragAnchors = useCallback((): void => {
    shiftDragAnchorRawValueRef.current = null;
    shiftDragAnchorOutputValueRef.current = null;
  }, []);

  useEffect(() => {
    latestOutputValueRef.current = clampedValue;
  }, [clampedValue]);

  useEffect(() => {
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
  }, [resetShiftDragAnchors]);

  return (
    <div
      className={mergeClassNames(
        'group inline-grid justify-items-center gap-2',
        knobWidthClassMap[size]
      )}
    >
      <label
        htmlFor={id}
        className="text-type-xs uppercase tracking-wide text-plugin-text-secondary"
      >
        {label}
      </label>

      <div className={mergeClassNames('relative', knobSizeClassMap[size])}>
        <input
          id={id}
          type="range"
          min={min}
          max={max}
          step={step}
          value={clampedValue}
          disabled={isDisabled}
          aria-busy={isLoading || undefined}
          aria-invalid={isError || undefined}
          aria-valuetext={formattedValue}
          data-state={state}
          data-plugin-state={pluginState}
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

            if (event.key === 'ArrowRight' || event.key === 'ArrowUp') {
              event.preventDefault();
              applyKeyboardDelta(
                isPrecisionMode ? keyboardSteps.precisionArrowStep : keyboardSteps.arrowStep
              );
              return;
            }

            if (event.key === 'ArrowLeft' || event.key === 'ArrowDown') {
              event.preventDefault();
              applyKeyboardDelta(
                isPrecisionMode ? -keyboardSteps.precisionArrowStep : -keyboardSteps.arrowStep
              );
              return;
            }

            if (event.key === 'PageUp') {
              event.preventDefault();
              applyKeyboardDelta(keyboardSteps.pageStep);
              return;
            }

            if (event.key === 'PageDown') {
              event.preventDefault();
              applyKeyboardDelta(-keyboardSteps.pageStep);
              return;
            }

            if (event.key === 'Home') {
              event.preventDefault();
              onChange(min);
              return;
            }

            if (event.key === 'End') {
              event.preventDefault();
              onChange(max);
            }
          }}
          onChange={(event): void => {
            const rawValue = Number.parseFloat(event.currentTarget.value);
            const nativeEvent = event.nativeEvent as {
              readonly shiftKey?: boolean;
              readonly getModifierState?: (keyArg: 'Shift') => boolean;
            };
            const isShiftActiveOnEvent = resolveShiftFromChangeEvent(nativeEvent);
            if (isPointerDragActiveRef.current && isShiftActiveOnEvent) {
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
          className={mergeClassNames(
            'peer absolute inset-0 z-20 h-full w-full cursor-pointer appearance-none rounded-full opacity-0',
            focusRingClass
          )}
        />

        <div
          className={mergeClassNames(
            'absolute inset-0 z-10 rounded-full border border-plugin-border bg-plugin-surface shadow-control',
            'peer-focus-visible:ring-2 peer-focus-visible:ring-accent-light peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-plugin-dark',
            getControlStateClass({ disabled: isDisabled, pluginState, state }),
            isError ? 'border-meter-clip' : ''
          )}
        >
          <span
            aria-hidden="true"
            className="pointer-events-none absolute inset-0"
            style={{ transform: `rotate(${angle}deg)` }}
          >
            <span
              aria-hidden="true"
              className="absolute left-1/2 top-1.5 h-2 w-2 -translate-x-1/2 rounded-full border border-plugin-dark bg-accent shadow-control"
            />
          </span>
        </div>

        {isLoading ? (
          <span
            aria-hidden="true"
            className="absolute inset-[30%] z-30 h-[40%] w-[40%] animate-spin rounded-full border border-plugin-text-secondary border-t-accent"
          />
        ) : null}
      </div>

      <div className="relative inline-flex w-full items-center justify-center gap-1">
        <span className="min-w-[72px] text-center font-mono text-type-sm tabular-nums text-plugin-text-primary">
          {formattedValue}
        </span>
        {badgeLabel ? (
          <span
            className={mergeClassNames(
              'rounded-sm border px-1 py-0.5 font-mono text-type-2xs',
              getStateBadgeClass(pluginState)
            )}
            aria-hidden="true"
          >
            {badgeLabel}
          </span>
        ) : null}
        <span
          aria-hidden="true"
          className="pointer-events-none absolute left-0 top-full mt-1 w-full px-1 text-center text-type-xs leading-tight text-plugin-text-secondary opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 motion-reduce:transition-none"
        >
          {SHIFT_PRECISION_HINT}
        </span>
      </div>
    </div>
  );
}
