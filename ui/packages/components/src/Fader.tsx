import { useCallback, useEffect, useRef, useState } from 'react';
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
  sm: 'w-full',
  md: 'w-full',
  lg: 'w-full',
};

const horizontalFootprintClassMap: Record<NonNullable<FaderProps['size']>, string> = {
  sm: 'h-8',
  md: 'h-9',
  lg: 'h-10',
};

const verticalLengthClassMap: Record<NonNullable<FaderProps['size']>, string> = {
  sm: 'h-[120px]',
  md: 'h-[160px]',
  lg: 'h-[220px]',
};

const verticalFootprintClassMap: Record<NonNullable<FaderProps['size']>, string> = {
  sm: 'w-9',
  md: 'w-10',
  lg: 'w-12',
};

const SHIFT_DRAG_PRECISION_DIVISOR = 12;
const HORIZONTAL_THUMB_DIAMETER_PX = 18;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function mapPointerClientYToValue(
  clientY: number,
  rect: Pick<DOMRectReadOnly, 'bottom' | 'height'>,
  min: number,
  max: number
): number {
  if (rect.height <= 0) {
    return min;
  }

  const normalized = clamp((rect.bottom - clientY) / rect.height, 0, 1);
  return min + normalized * (max - min);
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
  orientation = 'horizontal',
  pluginState,
  size = 'md',
  state = 'default',
  step = 0.001,
  unit,
  value,
}: Readonly<FaderProps>): React.JSX.Element {
  const activePointerIdRef = useRef<number | null>(null);
  const isPointerDragActiveRef = useRef(false);
  const isShiftPressedDuringDragRef = useRef(false);
  const shiftDragAnchorRawValueRef = useRef<number | null>(null);
  const shiftDragAnchorOutputValueRef = useRef<number | null>(null);
  const latestOutputValueRef = useRef(value);
  const [isPrecisionVisualActive, setIsPrecisionVisualActive] = useState(false);

  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isVertical = orientation === 'vertical';
  const keyboardSteps = getKeyboardSteps(min, max, step);
  const clampedValue = clamp(value, min, max);
  const range = Math.max(0, max - min);
  const normalizedValue = range > 0 ? clamp((clampedValue - min) / range, 0, 1) : 0;
  const horizontalThumbRadius = HORIZONTAL_THUMB_DIAMETER_PX / 2;
  const horizontalFillOffsetPx = horizontalThumbRadius * (1 - normalizedValue * 2);
  const horizontalFillWidth = `calc(${(normalizedValue * 100).toFixed(3)}% + ${horizontalFillOffsetPx.toFixed(3)}px)`;
  const verticalInputClass =
    'h-full w-2 [direction:rtl] [writing-mode:vertical-lr] [appearance:slider-vertical] [-webkit-appearance:slider-vertical]';

  const resetShiftDragAnchors = useCallback((): void => {
    shiftDragAnchorRawValueRef.current = null;
    shiftDragAnchorOutputValueRef.current = null;
  }, []);

  useEffect(() => {
    latestOutputValueRef.current = clampedValue;
  }, [clampedValue]);

  const emitPointerValue = useCallback(
    (pointerMappedValue: number, isShiftPrecisionMode: boolean): void => {
      const boundedPointerValue = clamp(pointerMappedValue, min, max);

      if (!isShiftPrecisionMode) {
        resetShiftDragAnchors();

        if (boundedPointerValue === latestOutputValueRef.current) {
          return;
        }

        latestOutputValueRef.current = boundedPointerValue;
        onChange(boundedPointerValue);
        return;
      }

      if (shiftDragAnchorRawValueRef.current === null) {
        shiftDragAnchorRawValueRef.current = boundedPointerValue;
        shiftDragAnchorOutputValueRef.current = latestOutputValueRef.current;
        return;
      }

      const anchorRawValue = shiftDragAnchorRawValueRef.current;
      const anchorOutputValue =
        shiftDragAnchorOutputValueRef.current ?? latestOutputValueRef.current;
      const precisionValue = clamp(
        anchorOutputValue + (boundedPointerValue - anchorRawValue) / SHIFT_DRAG_PRECISION_DIVISOR,
        min,
        max
      );

      if (precisionValue === latestOutputValueRef.current) {
        return;
      }

      latestOutputValueRef.current = precisionValue;
      onChange(precisionValue);
    },
    [max, min, onChange, resetShiftDragAnchors]
  );

  useEffect(() => {
    function handleWindowKeyDown(event: KeyboardEvent): void {
      if (event.key === 'Shift') {
        isShiftPressedDuringDragRef.current = true;
        if (isPointerDragActiveRef.current) {
          setIsPrecisionVisualActive(true);
        }
      }
    }

    function handleWindowKeyUp(event: KeyboardEvent): void {
      if (event.key === 'Shift') {
        isShiftPressedDuringDragRef.current = false;
        resetShiftDragAnchors();
        if (isPointerDragActiveRef.current) {
          setIsPrecisionVisualActive(false);
        }
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
    <div className="group inline-flex w-full flex-col items-center gap-2 align-middle">
      <label
        htmlFor={id}
        className="text-type-xs uppercase tracking-wide text-plugin-text-secondary"
      >
        {label}
      </label>

      <div
        className={mergeClassNames(
          'relative inline-flex items-center justify-center rounded-md border border-plugin-border bg-plugin-dark bg-gradient-to-b from-plugin-surface/60 to-plugin-dark p-2 shadow-control ring-1 ring-inset ring-plugin-border/60',
          isVertical
            ? `${verticalLengthClassMap[size]} ${verticalFootprintClassMap[size]}`
            : `${horizontalLengthClassMap[size]} ${horizontalFootprintClassMap[size]}`,
          getControlStateClass({ disabled: isDisabled, pluginState, state }),
          isPrecisionVisualActive ? 'ring-1 ring-accent/60' : '',
          isError ? 'border-meter-clip' : ''
        )}
      >
        {!isVertical ? (
          <div
            className="pointer-events-none absolute inset-x-2 top-1/2 z-0 -translate-y-1/2"
            aria-hidden="true"
          >
            <div className="h-2 w-full rounded-full bg-plugin-border" data-slot="horizontal-rail" />
            <div
              className="absolute left-0 top-1/2 h-1 w-full -translate-y-1/2 rounded-full bg-accent"
              style={{ width: horizontalFillWidth }}
              data-slot="horizontal-fill"
            />
          </div>
        ) : null}
        <input
          id={id}
          type="range"
          min={min}
          max={max}
          step={step}
          value={clampedValue}
          disabled={isDisabled}
          data-precision-control="true"
          data-precision-active={isPrecisionVisualActive ? 'true' : 'false'}
          onPointerDown={(event): void => {
            isPointerDragActiveRef.current = true;
            activePointerIdRef.current = event.pointerId;
            isShiftPressedDuringDragRef.current = event.shiftKey;
            setIsPrecisionVisualActive(event.shiftKey);
            resetShiftDragAnchors();

            if (!isVertical || isDisabled) {
              return;
            }

            event.preventDefault();
            event.currentTarget.setPointerCapture?.(event.pointerId);

            const mappedValue = mapPointerClientYToValue(
              event.clientY,
              event.currentTarget.getBoundingClientRect(),
              min,
              max
            );
            emitPointerValue(mappedValue, event.shiftKey);
          }}
          onPointerMove={(event): void => {
            if (!isVertical || isDisabled) {
              return;
            }

            if (!isPointerDragActiveRef.current || activePointerIdRef.current !== event.pointerId) {
              return;
            }

            event.preventDefault();

            const isShiftPrecisionMode = event.shiftKey || isShiftPressedDuringDragRef.current;
            const mappedValue = mapPointerClientYToValue(
              event.clientY,
              event.currentTarget.getBoundingClientRect(),
              min,
              max
            );

            setIsPrecisionVisualActive(isShiftPrecisionMode);
            emitPointerValue(mappedValue, isShiftPrecisionMode);
          }}
          onPointerUp={(event): void => {
            if (isVertical && activePointerIdRef.current === event.pointerId) {
              event.currentTarget.releasePointerCapture?.(event.pointerId);
            }

            activePointerIdRef.current = null;
            isPointerDragActiveRef.current = false;
            isShiftPressedDuringDragRef.current = false;
            setIsPrecisionVisualActive(false);
            resetShiftDragAnchors();
          }}
          onPointerCancel={(event): void => {
            if (isVertical && activePointerIdRef.current === event.pointerId) {
              event.currentTarget.releasePointerCapture?.(event.pointerId);
            }

            activePointerIdRef.current = null;
            isPointerDragActiveRef.current = false;
            isShiftPressedDuringDragRef.current = false;
            setIsPrecisionVisualActive(false);
            resetShiftDragAnchors();
          }}
          onKeyDown={(event): void => {
            if (isDisabled) {
              return;
            }

            if (event.key === 'Shift') {
              setIsPrecisionVisualActive(true);
              return;
            }

            const isPrecisionMode = isShiftPrecisionActive(event);

            if (event.key === 'ArrowUp' || event.key === 'ArrowRight') {
              event.preventDefault();
              setIsPrecisionVisualActive(isPrecisionMode);
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
              setIsPrecisionVisualActive(isPrecisionMode);
              const delta = isPrecisionMode
                ? -keyboardSteps.precisionArrowStep
                : -keyboardSteps.arrowStep;
              const nextValue = clamp(clampedValue + delta, min, max);
              if (nextValue !== clampedValue) {
                onChange(nextValue);
              }
            }
          }}
          onKeyUp={(event): void => {
            if (event.key === 'Shift') {
              setIsPrecisionVisualActive(false);
            }
          }}
          onBlur={(): void => {
            setIsPrecisionVisualActive(false);
          }}
          onChange={(event): void => {
            if (isVertical && isPointerDragActiveRef.current) {
              return;
            }

            const rawValue = Number.parseFloat(event.currentTarget.value);
            const nativeEvent = event.nativeEvent as {
              readonly shiftKey?: boolean;
              readonly getModifierState?: (keyArg: 'Shift') => boolean;
            };
            const isShiftActiveOnEvent = resolveShiftFromChangeEvent(nativeEvent);
            const mappedValue = rawValue;

            if (isPointerDragActiveRef.current && isShiftActiveOnEvent) {
              isShiftPressedDuringDragRef.current = isShiftActiveOnEvent;
            }

            const isShiftPrecisionMode = isPointerDragActiveRef.current
              ? isShiftPressedDuringDragRef.current
              : isShiftActiveOnEvent;

            setIsPrecisionVisualActive(isShiftPrecisionMode);

            if (!isShiftPrecisionMode) {
              resetShiftDragAnchors();
              latestOutputValueRef.current = mappedValue;
              onChange(mappedValue);
              return;
            }

            if (shiftDragAnchorRawValueRef.current === null) {
              shiftDragAnchorRawValueRef.current = mappedValue;
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
          aria-orientation={orientation}
          data-state={state}
          data-plugin-state={pluginState}
          className={mergeClassNames(
            'slider-thumb relative z-10 appearance-none rounded-full accent-accent ring-1 ring-inset ring-plugin-dark/80 [&::-moz-range-thumb:active]:bg-accent [&::-moz-range-thumb:hover]:bg-accent [&::-webkit-slider-thumb:active]:bg-accent [&::-webkit-slider-thumb:hover]:bg-accent',
            isPrecisionVisualActive ? 'cursor-zoom-in' : '',
            focusRingClass,
            isVertical ? `${verticalInputClass} bg-plugin-border` : 'h-2 w-full bg-transparent'
          )}
        />
      </div>

      <div className="relative inline-flex items-center gap-1">
        <span className="inline-block min-w-[6ch] text-right font-mono text-type-sm tabular-nums text-plugin-text-primary">
          {formatValue(clampedValue, unit)}
        </span>
        {badgeLabel ? (
          <span
            className={mergeClassNames(
              'rounded-sm border px-1 py-0.5 font-mono text-type-2xs leading-none',
              getStateBadgeClass(pluginState)
            )}
            aria-hidden="true"
          >
            {badgeLabel}
          </span>
        ) : null}
      </div>
    </div>
  );
}
