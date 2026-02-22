import type { ControlVisualState, PluginVisualState } from './types';
import { mergeClassNames } from './utils/classNames';
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

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
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
  pluginState,
  size = 'md',
  state = 'default',
  unit,
  value,
}: Readonly<KnobProps>): React.JSX.Element {
  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading;
  const clampedValue = clamp(value, min, max);
  const normalized = (clampedValue - min) / (max - min || 1);
  const angle = -135 + normalized * 270;
  const badgeLabel = getStateBadgeLabel(pluginState);

  return (
    <div className="inline-flex flex-col items-center gap-2">
      <label
        htmlFor={id}
        className="text-type-xs text-plugin-text-secondary uppercase tracking-wide"
      >
        {label}
      </label>

      <div className={mergeClassNames('relative', knobSizeClassMap[size])}>
        <input
          id={id}
          type="range"
          min={min}
          max={max}
          step="0.001"
          value={clampedValue}
          disabled={isDisabled}
          aria-busy={isLoading || undefined}
          aria-invalid={isError || undefined}
          data-state={state}
          data-plugin-state={pluginState}
          onChange={(event): void => {
            onChange(Number.parseFloat(event.currentTarget.value));
          }}
          className="peer absolute inset-0 z-20 h-full w-full cursor-pointer appearance-none rounded-full opacity-0"
        />

        <div
          className={mergeClassNames(
            'shadow-control absolute inset-0 z-10 rounded-full border border-plugin-border bg-plugin-surface',
            'peer-focus-visible:ring-2 peer-focus-visible:ring-accent-light peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-plugin-dark',
            getControlStateClass({ disabled: isDisabled, pluginState, state }),
            isError ? 'border-meter-clip' : ''
          )}
        >
          <span
            aria-hidden="true"
            className="absolute left-1/2 top-1 h-[40%] w-0.5 -translate-x-1/2 rounded-full bg-accent"
            style={{ transform: `translateX(-50%) rotate(${angle}deg)` }}
          />
        </div>

        {isLoading ? (
          <span
            aria-hidden="true"
            className="border-plugin-text-secondary absolute inset-[30%] z-30 h-[40%] w-[40%] animate-spin rounded-full border border-t-accent"
          />
        ) : null}
      </div>

      <div className="inline-flex items-center gap-1">
        <span className="text-type-sm text-plugin-text-primary font-mono tabular-nums">
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
      </div>
    </div>
  );
}
