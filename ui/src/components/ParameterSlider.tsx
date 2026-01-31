/**
 * ParameterSlider - Slider control for float parameters
 */

import React from 'react';
import { useParameter } from '@vstkit/ipc';
import './ParameterSlider.css';

interface ParameterSliderProps {
  id: string;
}

export function ParameterSlider({ id }: ParameterSliderProps): React.JSX.Element {
  const { param, setValue, isLoading, error } = useParameter(id);

  if (isLoading) {
    return <div className="parameter-slider loading">Loading {id}...</div>;
  }

  if (error || !param) {
    return (
      <div className="parameter-slider error">Error: {error?.message || 'Parameter not found'}</div>
    );
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    const value = parseFloat(e.target.value);
    setValue(value).catch((err) => {
      console.error('Failed to set parameter:', err);
    });
  };

  // Format display value
  const displayValue = param.unit
    ? `${(param.value * 100).toFixed(1)}${param.unit === '%' ? param.unit : ' ' + param.unit}`
    : param.value.toFixed(3);

  return (
    <div className="parameter-slider">
      <div className="parameter-header">
        <label htmlFor={`slider-${id}`}>{param.name}</label>
        <span className="parameter-value">{displayValue}</span>
      </div>
      <input
        id={`slider-${id}`}
        type="range"
        min="0"
        max="1"
        step="0.001"
        value={param.value}
        onChange={handleChange}
        className="slider"
      />
    </div>
  );
}
