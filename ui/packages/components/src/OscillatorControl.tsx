/**
 * OscillatorControl - Displays oscillator signal status and oscillator-specific controls.
 */

import React from 'react';
import { logger, useMeterFrame, useParameter, useHasProcessor } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';
import { ParameterSlider } from './ParameterSlider';

const SIGNAL_THRESHOLD = 1e-4;
const OSCILLATOR_ENABLED_PARAM_ID = 'oscillator_enabled' as ParameterId;
const OSCILLATOR_FREQUENCY_PARAM_ID = 'oscillator_frequency' as ParameterId;
const OSCILLATOR_LEVEL_PARAM_ID = 'oscillator_level' as ParameterId;

interface OscilloscopeProps {
  hideWhenNotInSignalChain?: boolean;
}

export function OscillatorControl(props: Readonly<OscilloscopeProps>): React.JSX.Element | null {
  const meterFrame = useMeterFrame(100);
  const {
    param: oscillatorEnabled,
    setValue: setOscillatorEnabled,
    isLoading: isOscillatorLoading,
    error: oscillatorError,
  } = useParameter(OSCILLATOR_ENABLED_PARAM_ID);

  const hasProcessorInSignalChain = useHasProcessor('oscillator');

  if (props.hideWhenNotInSignalChain && !hasProcessorInSignalChain) {
    return null;
  }

  const oscillatorPeak = Math.max(meterFrame?.peak_l ?? 0, meterFrame?.peak_r ?? 0);
  const isProducing = oscillatorPeak > SIGNAL_THRESHOLD;
  const isOn = Boolean(oscillatorEnabled?.value ?? false);
  const signalStatusLabel = isProducing ? 'Producing' : 'No signal';
  const outputStateLabel = isOn ? 'On' : 'Off';
  const signalStatusClassName = isProducing
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
        parameterId: OSCILLATOR_ENABLED_PARAM_ID,
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
        <span className="text-sm font-semibold text-gray-200">Oscillator signal</span>
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
        <ParameterSlider id={OSCILLATOR_FREQUENCY_PARAM_ID} />
        <ParameterSlider id={OSCILLATOR_LEVEL_PARAM_ID} />
      </div>
    </div>
  );
}
