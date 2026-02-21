/**
 * ParameterSlider - Slider control for float parameters
 */

import React from 'react';
import { focusRingClass, interactionStateClass, surfaceCardClass } from './utils/classNames';

export interface ParameterSliderProps {
  readonly id: string;
  readonly name: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly unit?: string;
  readonly disabled?: boolean;
  readonly onChange: (value: number) => void | Promise<void>;
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

export function ParameterSlider({
  id,
  name,
  value,
  min,
  max,
  unit,
  disabled = false,
  onChange,
}: Readonly<ParameterSliderProps>): React.JSX.Element {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    const nextValue = Number.parseFloat(e.currentTarget.value);
    void onChange(nextValue);
  };

  const displayValue = formatParameterValue(value, unit);

  return (
    <div data-testid={`param-${id}`} className={`mb-4 ${surfaceCardClass}`}>
      <div className="mb-2 flex items-center justify-between">
        <label
          data-testid={`param-${id}-label`}
          htmlFor={`slider-${id}`}
          className="font-semibold text-gray-200"
        >
          {name}
        </label>
        <span data-testid={`param-${id}-value`} className="font-mono text-sm text-accent">
          {displayValue}
        </span>
      </div>
      <input
        data-testid={`param-${id}-slider`}
        id={`slider-${id}`}
        type="range"
        min={min}
        max={max}
        step="0.001"
        value={value}
        onChange={handleChange}
        disabled={disabled}
        className={`slider-thumb h-1.5 w-full appearance-none rounded-sm bg-plugin-border ${focusRingClass} ${interactionStateClass}`}
      />
    </div>
  );
}
