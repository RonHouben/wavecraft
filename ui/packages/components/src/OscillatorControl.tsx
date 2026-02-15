/**
 * OscillatorControl - Displays oscillator signal status and a dedicated on/off toggle.
 */

import React from 'react';

interface OscillatorControlProps {
  readonly isProducing: boolean;
  readonly isOn: boolean;
  readonly onToggle: () => void;
}

export function OscillatorControl({
  isProducing,
  isOn,
  onToggle,
}: OscillatorControlProps): React.JSX.Element {
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
          onClick={onToggle}
          aria-label="Toggle oscillator output"
          aria-pressed={isOn}
        >
          <span
            className={`absolute top-[3px] h-5 w-5 rounded-full bg-white transition-[left] duration-200 ${
              isOn ? 'left-[27px]' : 'left-[3px]'
            }`}
          ></span>
        </button>
      </div>
    </div>
  );
}
