import type { ControlVisualState, PluginVisualState } from '../types';
import { mergeClassNames } from './classNames';

export interface ControlStateOptions {
  readonly state?: ControlVisualState;
  readonly pluginState?: PluginVisualState;
  readonly disabled?: boolean;
}

export function getControlStateClass({
  state = 'default',
  pluginState,
  disabled = false,
}: Readonly<ControlStateOptions>): string {
  const isDisabled = disabled || state === 'disabled';

  if (isDisabled) {
    return 'cursor-not-allowed opacity-50';
  }

  const baseInteractiveClass =
    'motion-safe:transition-all motion-safe:duration-150 hover:brightness-110 active:scale-[0.98]';

  const baselineStateClass =
    state === 'loading'
      ? 'pointer-events-none cursor-wait'
      : state === 'error'
        ? 'border border-meter-clip text-meter-clip'
        : state === 'hover'
          ? 'brightness-110'
          : state === 'focus'
            ? 'ring-2 ring-accent-light ring-offset-2 ring-offset-plugin-dark'
            : state === 'active'
              ? 'scale-[0.98] brightness-95'
              : '';

  const pluginStateClass =
    pluginState === 'bypassed'
      ? 'saturate-0'
      : pluginState === 'armed'
        ? 'ring-1 ring-state-warning/70'
        : pluginState === 'mapped'
          ? 'ring-1 ring-state-info/70'
          : '';

  return mergeClassNames(baseInteractiveClass, baselineStateClass, pluginStateClass);
}

export function getStateBadgeLabel(pluginState?: PluginVisualState): string | null {
  if (!pluginState) {
    return null;
  }

  if (pluginState === 'bypassed') {
    return 'BYP';
  }

  if (pluginState === 'armed') {
    return 'ARM';
  }

  return 'MAP';
}

export function getStateBadgeClass(pluginState?: PluginVisualState): string {
  if (pluginState === 'bypassed') {
    return 'border-plugin-border text-plugin-text-muted';
  }

  if (pluginState === 'armed') {
    return 'border-state-warning/60 text-state-warning';
  }

  if (pluginState === 'mapped') {
    return 'border-state-info/60 text-state-info';
  }

  return 'border-plugin-border text-plugin-text-secondary';
}
