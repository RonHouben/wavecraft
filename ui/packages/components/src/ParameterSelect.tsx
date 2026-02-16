/**
 * ParameterSelect - Dropdown control for enum parameters.
 */

import React from 'react';
import { useParameter, logger } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

export interface ParameterSelectProps {
  /** Parameter ID to bind to. */
  readonly id: ParameterId;
}

export function ParameterSelect({ id }: Readonly<ParameterSelectProps>): React.JSX.Element {
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

  const variants = param.variants ?? [];
  const currentIndex = typeof param.value === 'number' ? param.value : 0;

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>): void => {
    const nextValue = Number(event.currentTarget.value);
    setValue(nextValue).catch((err) => {
      logger.error('Failed to set enum parameter', { error: err, parameterId: id });
    });
  };

  return (
    <div className="mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <div className="mb-2 flex items-center justify-between">
        <label htmlFor={`select-${id}`} className="font-semibold text-gray-200">
          {param.name}
        </label>
      </div>

      <select
        id={`select-${id}`}
        value={currentIndex}
        onChange={handleChange}
        className="w-full rounded border border-plugin-border bg-plugin-dark px-3 py-2 text-sm text-gray-200 outline-none focus:border-accent"
      >
        {variants.map((label, index) => (
          <option key={`${id}-${label}-${index}`} value={index}>
            {label}
          </option>
        ))}
      </select>
    </div>
  );
}
