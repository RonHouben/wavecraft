/**
 * OscillatorControl - Displays oscillator signal status and oscillator-specific controls.
 */

import React from 'react';
import { logger, useMeterFrame, useParameter } from '@wavecraft/core';
import { ParameterSlider } from './ParameterSlider';

const SIGNAL_THRESHOLD = 1e-4;

export function OscillatorControl(): React.JSX.Element {
  const meterFrame = useMeterFrame(100);
  const {
    param: oscillatorEnabled,
    setValue: setOscillatorEnabled,
    isLoading: isOscillatorLoading,
    error: oscillatorError,
  } = useParameter('oscillator_enabled');

  const oscillatorPeak = Math.max(meterFrame?.peak_l ?? 0, meterFrame?.peak_r ?? 0);
  const isProducing = oscillatorPeak > SIGNAL_THRESHOLD;
  const isOn = (oscillatorEnabled?.value ?? 0) >= 0.5;

  const handleToggle = (): void => {
    if (!oscillatorEnabled) {
      return;
    }

    const newValue = isOn ? 0 : 1;
    setOscillatorEnabled(newValue).catch((error) => {
      logger.error('Failed to toggle oscillator output', {
        error,
        parameterId: 'oscillator_enabled',
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
        <span
          className={`rounded px-2 py-1 text-xs font-semibold ${
            isProducing ? 'bg-green-900/30 text-green-400' : 'bg-yellow-900/30 text-yellow-400'
          }`}
        >
          {isProducing ? 'Producing' : 'No signal'}
        </span>
      </div>

      <div className="flex items-center justify-between gap-3">
        <span className="text-sm text-gray-300">Oscillator output: {isOn ? 'On' : 'Off'}</span>
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
        <ParameterSlider id="oscillator_frequency" />
        <ParameterSlider id="oscillator_level" />
      </div>
    </div>
  );
}
