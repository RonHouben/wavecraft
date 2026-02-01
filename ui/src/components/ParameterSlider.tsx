/**
 * ParameterSlider - Slider control for float parameters
 */

import React from 'react';
import { useParameter } from '@vstkit/ipc';

interface ParameterSliderProps {
  readonly id: string;
}

export function ParameterSlider({ id }: ParameterSliderProps): React.JSX.Element {
  const { param, setValue, isLoading, error } = useParameter(id);

  if (isLoading) {
    return (
      <div className="mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4 italic text-gray-500">
        Loading {id}...
      </div>
    );
  }

  if (error || !param) {
    return (
      <div className="mb-4 rounded-lg border border-red-400 bg-plugin-surface p-4 text-red-400">
        Error: {error?.message || 'Parameter not found'}
      </div>
    );
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    const value = Number.parseFloat(e.target.value);
    setValue(value).catch((err) => {
      console.error('Failed to set parameter:', err);
    });
  };

  // Format display value
  const unitSuffix = param.unit === '%' ? param.unit : ` ${param.unit}`;
  const displayValue = param.unit
    ? `${(param.value * 100).toFixed(1)}${unitSuffix}`
    : param.value.toFixed(3);

  return (
    <div
      data-testid={`param-${id}`}
      className="mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4"
    >
      <div className="mb-2 flex items-center justify-between">
        <label
          data-testid={`param-${id}-label`}
          htmlFor={`slider-${id}`}
          className="font-semibold text-gray-200"
        >
          {param.name}
        </label>
        <span data-testid={`param-${id}-value`} className="font-mono text-sm text-accent">
          {displayValue}
        </span>
      </div>
      <input
        data-testid={`param-${id}-slider`}
        id={`slider-${id}`}
        type="range"
        min="0"
        max="1"
        step="0.001"
        value={param.value}
        onChange={handleChange}
        className="slider-thumb h-1.5 w-full appearance-none rounded-sm bg-plugin-border outline-none"
      />
    </div>
  );
}
