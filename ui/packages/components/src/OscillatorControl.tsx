/**
 * OscillatorControl - Displays oscillator signal status and oscillator-specific controls.
 */

import React from 'react';
import {
  logger,
  useMeterFrame,
  useParameter,
  useHasProcessor,
  useAllParameters,
} from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';
import { ParameterSelect } from './ParameterSelect';
import { ParameterSlider } from './ParameterSlider';

const SIGNAL_THRESHOLD = 1e-4;
const OSCILLATOR_ENABLED_PARAM_ID = 'oscillator_enabled' as const;
const OSCILLATOR_WAVEFORM_PARAM_ID = 'oscillator_waveform' as const;
const OSCILLATOR_FREQUENCY_PARAM_ID = 'oscillator_frequency' as const;
const OSCILLATOR_LEVEL_PARAM_ID = 'oscillator_level' as const;

interface OscillatorParamIdSet {
  enabled: ParameterId;
  waveform: ParameterId;
  frequency: ParameterId;
  level: ParameterId;
}

const CANONICAL_OSCILLATOR_PARAM_IDS: OscillatorParamIdSet = {
  enabled: OSCILLATOR_ENABLED_PARAM_ID,
  waveform: OSCILLATOR_WAVEFORM_PARAM_ID,
  frequency: OSCILLATOR_FREQUENCY_PARAM_ID,
  level: OSCILLATOR_LEVEL_PARAM_ID,
};

const LEGACY_OSCILLATOR_PARAM_ALIASES: Record<keyof OscillatorParamIdSet, readonly string[]> = {
  enabled: ['param_0'],
  waveform: ['param_1'],
  frequency: ['param_2'],
  level: ['param_3'],
};

function suffixFromCanonicalId(id: string): string {
  return id.startsWith('oscillator_') ? id.slice('oscillator_'.length) : id;
}

function resolveOscillatorParamId(
  availableIds: ReadonlySet<string>,
  canonicalId: string,
  legacyAliases: readonly string[]
): ParameterId {
  if (availableIds.has(canonicalId)) {
    return canonicalId as ParameterId;
  }

  const explicitAlias = legacyAliases.find((alias) => availableIds.has(alias));
  if (explicitAlias) {
    return explicitAlias as ParameterId;
  }

  const suffix = suffixFromCanonicalId(canonicalId);
  const suffixMatch = Array.from(availableIds).find((id) => id.endsWith(`_${suffix}`));
  if (suffixMatch) {
    return suffixMatch as ParameterId;
  }

  return canonicalId as ParameterId;
}

function resolveOscillatorParamIds(parameterIds: readonly string[]): OscillatorParamIdSet {
  const availableIds = new Set(parameterIds);

  return {
    enabled: resolveOscillatorParamId(
      availableIds,
      CANONICAL_OSCILLATOR_PARAM_IDS.enabled,
      LEGACY_OSCILLATOR_PARAM_ALIASES.enabled
    ),
    waveform: resolveOscillatorParamId(
      availableIds,
      CANONICAL_OSCILLATOR_PARAM_IDS.waveform,
      LEGACY_OSCILLATOR_PARAM_ALIASES.waveform
    ),
    frequency: resolveOscillatorParamId(
      availableIds,
      CANONICAL_OSCILLATOR_PARAM_IDS.frequency,
      LEGACY_OSCILLATOR_PARAM_ALIASES.frequency
    ),
    level: resolveOscillatorParamId(
      availableIds,
      CANONICAL_OSCILLATOR_PARAM_IDS.level,
      LEGACY_OSCILLATOR_PARAM_ALIASES.level
    ),
  };
}

interface OscilloscopeProps {
  hideWhenNotInSignalChain?: boolean;
}

export function OscillatorControl(props: Readonly<OscilloscopeProps>): React.JSX.Element | null {
  const meterFrame = useMeterFrame(100);
  const { params: allParameters } = useAllParameters();
  const oscillatorParamIds = React.useMemo(
    () => resolveOscillatorParamIds(allParameters.map((param) => param.id)),
    [allParameters]
  );

  const {
    param: oscillatorEnabled,
    setValue: setOscillatorEnabled,
    isLoading: isOscillatorLoading,
    error: oscillatorError,
  } = useParameter(oscillatorParamIds.enabled);

  const hasProcessorInSignalChain = useHasProcessor('oscillator');

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const oscillatorPeak = Math.max(meterFrame?.peak_l ?? 0, meterFrame?.peak_r ?? 0);
  const hasOutputSignal = oscillatorPeak > SIGNAL_THRESHOLD;
  const isOn = Boolean(oscillatorEnabled?.value ?? false);
  const signalStatusLabel = !isOn ? 'Off' : hasOutputSignal ? 'Signal at output' : 'On (no output)';
  const outputStateLabel = isOn ? 'On' : 'Off';
  const signalStatusClassName = !isOn
    ? 'bg-gray-700/50 text-gray-300'
    : hasOutputSignal
      ? 'bg-green-900/30 text-green-400'
      : 'bg-yellow-900/30 text-yellow-400';

  const handleToggle = (): void => {
    if (!oscillatorEnabled) {
      return;
    }

    const newValue = !isOn;
    setOscillatorEnabled(newValue).catch((error) => {
      logger.error('Failed to toggle oscillator output', {
        error,
        parameterId: oscillatorParamIds.enabled,
      });
    });
  };

  const toggleDisabled = isOscillatorLoading || !oscillatorEnabled || Boolean(oscillatorError);

  return (
    <div
      className="mb-3 rounded-md border border-plugin-border bg-plugin-dark p-3"
      data-testid="oscillator-control"
    >
      <div className="mb-2 flex items-center justify-between gap-3">
        <span className="text-sm font-semibold text-gray-200">Output signal (post-chain)</span>
        <span className={`rounded px-2 py-1 text-xs font-semibold ${signalStatusClassName}`}>
          {signalStatusLabel}
        </span>
      </div>

      <div className="flex items-center justify-between gap-3">
        <span className="text-sm text-gray-300">Oscillator output: {outputStateLabel}</span>
        <button
          type="button"
          className={`relative h-[26px] w-[50px] cursor-pointer rounded-full border-none outline-none transition-colors duration-200 ${
            isOn ? 'bg-accent hover:bg-accent-light' : 'bg-gray-600 hover:bg-gray-500'
          }`}
          onClick={handleToggle}
          aria-label="Toggle oscillator output"
          aria-pressed={isOn}
          disabled={toggleDisabled}
        >
          <span
            className={`absolute top-[3px] h-5 w-5 rounded-full bg-white transition-[left] duration-200 ${
              isOn ? 'left-[27px]' : 'left-[3px]'
            }`}
          />
        </button>
      </div>

      {oscillatorError ? (
        <p className="mt-3 text-sm text-red-400">Error: {oscillatorError.message}</p>
      ) : null}

      <div className="mt-3">
        <ParameterSelect id={oscillatorParamIds.waveform} />
        <ParameterSlider id={oscillatorParamIds.frequency} />
        <ParameterSlider id={oscillatorParamIds.level} />
      </div>
    </div>
  );
}
