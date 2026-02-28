import { useId, type ReactNode, type SelectHTMLAttributes } from 'react';
import type { ControlVisualState, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

type SelectValue = string | number;

export interface SelectOption<T extends SelectValue> {
  readonly label: ReactNode;
  readonly value: T;
  readonly disabled?: boolean;
}

type NativeSelectProps = Omit<
  SelectHTMLAttributes<HTMLSelectElement>,
  'children' | 'disabled' | 'id' | 'onChange' | 'size' | 'value'
>;

export interface SelectProps<T extends SelectValue = string> extends NativeSelectProps {
  readonly value: T;
  readonly options: readonly SelectOption<T>[];
  readonly onChange: (value: T) => void;
  readonly className?: string;
  readonly disabled?: boolean;
  readonly id?: string;
  readonly label?: ReactNode;
  readonly pluginState?: PluginVisualState;
  readonly size?: 'sm' | 'md' | 'lg';
  readonly state?: ControlVisualState;
}

const selectSizeClassMap: Record<NonNullable<SelectProps['size']>, string> = {
  sm: 'h-8 px-2.5 text-type-xs',
  md: 'h-9 px-3 text-type-sm',
  lg: 'h-10 px-4 text-type-md',
};

function getSelectedOptionIndex<T extends SelectValue>(
  options: readonly SelectOption<T>[],
  value: T
): number {
  return options.findIndex((option) => option.value === value);
}

export function Select<T extends SelectValue = string>({
  className,
  disabled = false,
  id,
  label,
  onChange,
  options,
  pluginState,
  size = 'md',
  state = 'default',
  value,
  ...rest
}: Readonly<SelectProps<T>>): React.JSX.Element {
  const generatedId = useId();
  const selectId = id ?? `${generatedId}-select`;
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const selectedOptionIndex = getSelectedOptionIndex(options, value);

  return (
    <div className="inline-flex gap-1.5">
      {label || badgeLabel ? (
        <div
          className={mergeClassNames(
            'flex items-center gap-2',
            badgeLabel ? 'justify-between' : 'justify-start'
          )}
        >
          {label ? (
            <label
              htmlFor={selectId}
              className="text-type-sm font-medium text-plugin-text-secondary"
            >
              {label}
            </label>
          ) : null}

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
      ) : null}

      <select
        id={selectId}
        value={selectedOptionIndex >= 0 ? String(selectedOptionIndex) : ''}
        onChange={(event): void => {
          const nextIndex = Number.parseInt(event.currentTarget.value, 10);
          const nextOption = options[nextIndex];

          if (!nextOption || nextOption.disabled || isDisabled) {
            return;
          }

          onChange(nextOption.value);
        }}
        disabled={isDisabled}
        aria-busy={isLoading || undefined}
        aria-invalid={isError || undefined}
        data-state={state}
        data-plugin-state={pluginState}
        className={mergeClassNames(
          'appearance-none rounded-md border border-plugin-border bg-plugin-surface text-plugin-text-primary shadow-control',
          selectSizeClassMap[size],
          focusRingClass,
          getControlStateClass({ disabled: isDisabled, pluginState, state }),
          className
        )}
        {...rest}
      >
        {options.map((option, index) => (
          <option
            key={`${selectId}-${String(option.value)}-${index}`}
            value={String(index)}
            disabled={option.disabled}
          >
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
}
