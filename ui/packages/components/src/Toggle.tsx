import { useId, type ReactNode } from 'react';
import type { ControlVisualState, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

export interface ToggleProps {
  readonly checked: boolean;
  readonly disabled?: boolean;
  readonly id?: string;
  readonly label?: ReactNode;
  readonly onChange: (checked: boolean) => void;
  readonly pluginState?: PluginVisualState;
  readonly size?: 'sm' | 'md' | 'lg';
  readonly state?: ControlVisualState;
}

interface ToggleSizeStyle {
  readonly track: string;
  readonly thumb: string;
  readonly thumbOffsetOn: string;
}

const toggleSizeClassMap: Record<NonNullable<ToggleProps['size']>, ToggleSizeStyle> = {
  sm: {
    track: 'h-4 w-7',
    thumb: 'h-3 w-3 top-0.5',
    thumbOffsetOn: 'left-[14px]',
  },
  md: {
    track: 'h-5 w-9',
    thumb: 'h-4 w-4 top-0.5',
    thumbOffsetOn: 'left-[18px]',
  },
  lg: {
    track: 'h-6 w-11',
    thumb: 'h-5 w-5 top-0.5',
    thumbOffsetOn: 'left-[22px]',
  },
};

export function Toggle({
  checked,
  disabled = false,
  id,
  label,
  onChange,
  pluginState,
  size = 'md',
  state = 'default',
}: Readonly<ToggleProps>): React.JSX.Element {
  const generatedLabelId = useId();
  const labelId = label ? `${id ?? generatedLabelId}-label` : undefined;
  const badgeLabel = getStateBadgeLabel(pluginState);
  const sizeStyle = toggleSizeClassMap[size];
  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const resolvedChecked = pluginState === 'bypassed' ? false : checked;

  return (
    <div className="inline-flex items-center gap-2">
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={resolvedChecked}
        aria-busy={isLoading || undefined}
        aria-invalid={isError || undefined}
        aria-labelledby={labelId}
        disabled={isDisabled}
        data-state={state}
        data-plugin-state={pluginState}
        onClick={(): void => {
          onChange(!resolvedChecked);
        }}
        className={mergeClassNames(
          'relative rounded-full border border-plugin-border',
          sizeStyle.track,
          focusRingClass,
          getControlStateClass({ disabled: isDisabled, pluginState, state }),
          resolvedChecked ? 'bg-accent' : 'bg-plugin-surface',
          isError ? 'border-meter-clip bg-meter-clip/10' : ''
        )}
      >
        <span
          className={mergeClassNames(
            'bg-plugin-text-primary absolute left-0.5 rounded-full motion-safe:transition-all motion-safe:duration-150',
            sizeStyle.thumb,
            resolvedChecked ? sizeStyle.thumbOffsetOn : ''
          )}
        />
      </button>

      {label ? (
        <span id={labelId} className="text-type-sm text-plugin-text-secondary">
          {label}
        </span>
      ) : null}

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
  );
}
