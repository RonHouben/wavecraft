/**
 * ParameterSlider - Slider control for float parameters
 */

import React from 'react';
import { useParameter, logger } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

interface ParameterSliderProps {
  readonly id: ParameterId;
}

function formatParameterValue(value: number, unit?: string): string {
  if (!unit) {
    return value.toFixed(3);
  }

  if (unit === '%') {
    return `${(value * 100).toFixed(1)}%`;
  }

  return `${value.toFixed(1)} ${unit}`;
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
    const message = error?.message || 'Parameter not found';

    return (
      <div className="mb-4 rounded-lg border border-red-400 bg-plugin-surface p-4 text-red-400">
        Error: {message}
      </div>
    );
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    const nextValue = Number.parseFloat(e.currentTarget.value);
    setValue(nextValue).catch((err) => {
      logger.error('Failed to set parameter', { error: err, parameterId: id });
    });
  };
  const displayValue = formatParameterValue(param.value, param.unit);

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
        min={param.min}
        max={param.max}
        step="0.001"
        value={param.value}
        onChange={handleChange}
        className="slider-thumb h-1.5 w-full appearance-none rounded-sm bg-plugin-border outline-none"
      />
    </div>
  );
}
