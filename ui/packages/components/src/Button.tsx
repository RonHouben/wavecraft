import type { ButtonHTMLAttributes, ReactNode } from 'react';
import type { ControlVisualState, PluginVisualState } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import {
  getControlStateClass,
  getStateBadgeClass,
  getStateBadgeLabel,
} from './utils/controlStates';

type NativeButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'>;

export interface ButtonProps extends NativeButtonProps {
  readonly children: ReactNode;
  readonly size?: 'sm' | 'md' | 'lg';
  readonly state?: ControlVisualState;
  readonly pluginState?: PluginVisualState;
  readonly iconLeft?: ReactNode;
  readonly active?: boolean;
  readonly isActive?: boolean;
  readonly pressed?: boolean;
}

const buttonSizeClassMap: Record<NonNullable<ButtonProps['size']>, string> = {
  sm: 'min-w-14 px-2.5 h-6 text-type-xs',
  md: 'min-w-[72px] px-3 h-8 text-type-sm',
  lg: 'min-w-[88px] px-4 h-10 text-type-md',
};

export function Button({
  active: activeProp,
  children,
  className,
  disabled = false,
  iconLeft,
  isActive,
  onClick,
  pluginState,
  pressed,
  size = 'md',
  state = 'default',
  type = 'button',
  ...rest
}: Readonly<ButtonProps>): React.JSX.Element {
  const badgeLabel = getStateBadgeLabel(pluginState);
  const isLoading = state === 'loading';
  const isError = state === 'error';
  const isDisabled = disabled || isLoading || state === 'disabled';
  const hasActiveState =
    activeProp !== undefined || isActive !== undefined || pressed !== undefined;
  const active = activeProp ?? isActive ?? pressed ?? false;

  return (
    <button
      type={type}
      className={mergeClassNames(
        'inline-flex items-center justify-center gap-1 rounded-md border border-plugin-border bg-plugin-surface text-plugin-text-primary shadow-control',
        buttonSizeClassMap[size],
        focusRingClass,
        getControlStateClass({ disabled: isDisabled, pluginState, state }),
        active
          ? 'border-accent-light bg-gradient-to-b from-accent/30 to-accent/15 font-semibold text-accent shadow-md ring-1 ring-inset ring-accent/45'
          : '',
        isError ? 'border-meter-clip bg-meter-clip/10 text-meter-clip' : '',
        className
      )}
      onClick={onClick}
      disabled={isDisabled}
      aria-busy={isLoading || undefined}
      aria-invalid={isError || undefined}
      aria-pressed={hasActiveState ? active : undefined}
      data-state={state}
      data-plugin-state={pluginState}
      data-active={hasActiveState ? String(active) : undefined}
      {...rest}
    >
      {isLoading ? (
        <span
          aria-hidden="true"
          className="h-3.5 w-3.5 animate-spin rounded-full border border-plugin-text-secondary border-t-accent"
        />
      ) : (
        iconLeft
      )}

      <span>{children}</span>

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
    </button>
  );
}
