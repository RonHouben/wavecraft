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
  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isVertical = orientation === 'vertical';

  return (
    <div className="inline-flex flex-col items-center gap-2">
      <label
        htmlFor={id}
        className="text-type-xs text-plugin-text-secondary uppercase tracking-wide"
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
          value={value}
          disabled={isDisabled}
          onChange={(event): void => {
            onChange(Number.parseFloat(event.currentTarget.value));
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

      <div className="inline-flex items-center gap-1">
        <span className="text-type-sm text-plugin-text-primary font-mono tabular-nums">
          {formatValue(value, unit)}
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
      </div>
    </div>
  );
}
