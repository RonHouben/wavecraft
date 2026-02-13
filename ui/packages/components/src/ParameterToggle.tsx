/**
 * ParameterToggle - Toggle control for boolean parameters
 */

import React from 'react';
import { useParameter, logger } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

interface ParameterToggleProps {
  readonly id: ParameterId;
}

export function ParameterToggle({ id }: ParameterToggleProps): React.JSX.Element {
  const { param, setValue, isLoading, error } = useParameter(id);

  if (isLoading) {
    return (
      <div className="mb-4 flex items-center justify-between rounded-lg border border-plugin-border bg-plugin-surface p-4 italic text-gray-500">
        Loading {id}...
      </div>
    );
  }

  if (error || !param) {
    return (
      <div className="mb-4 flex items-center justify-between rounded-lg border border-red-400 bg-plugin-surface p-4 text-red-400">
        Error: {error?.message || 'Parameter not found'}
      </div>
    );
  }

  const isOn = param.value >= 0.5;

  const handleToggle = (): void => {
    const newValue = isOn ? 0 : 1;
    setValue(newValue).catch((err) => {
      logger.error('Failed to set parameter', { error: err, parameterId: id });
    });
  };

  return (
    <div className="mb-4 flex items-center justify-between rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <label htmlFor={`toggle-${id}`} className="font-semibold text-gray-200">
        {param.name}
      </label>
      <button
        id={`toggle-${id}`}
        className={`relative h-[26px] w-[50px] cursor-pointer rounded-full border-none outline-none transition-colors duration-200 ${
          isOn ? 'bg-accent hover:bg-accent-light' : 'bg-gray-600 hover:bg-gray-500'
        }`}
        onClick={handleToggle}
        aria-pressed={isOn}
      >
        <span
          className={`absolute top-[3px] h-5 w-5 rounded-full bg-white transition-[left] duration-200 ${
            isOn ? 'left-[27px]' : 'left-[3px]'
          }`}
        ></span>
      </button>
    </div>
  );
}
