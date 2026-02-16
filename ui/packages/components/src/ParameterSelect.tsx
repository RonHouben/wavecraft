/**
 * ParameterSelect - Dropdown control for enum parameters.
 */

import React, { useEffect } from 'react';
import { useParameter, logger } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

export interface ParameterSelectProps {
  /** Parameter ID to bind to. */
  readonly id: ParameterId;
}

export function ParameterSelect({ id }: Readonly<ParameterSelectProps>): React.JSX.Element {
  const { param, setValue, isLoading, error } = useParameter(id);

  useEffect(() => {
    const hasNoVariants = !!param && !param.variants?.length;

    if (hasNoVariants) {
      logger.warn('Enum parameter has no variants', {
        parameterId: id,
      });

    }
  }, [param, id]);

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

  const currentIndex = typeof param.value === 'number' ? param.value : 0;
  const variantOptions = (param.variants ?? []).map((label, index) => ({ value: index, label }));
  const hasNoVariants = variantOptions.length === 0;
  const helperTextId = hasNoVariants ? `select-${id}-helper` : undefined;

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
        disabled={hasNoVariants}
        aria-describedby={helperTextId}
        className="w-full rounded border border-plugin-border bg-plugin-dark px-3 py-2 text-sm text-gray-200 outline-none focus:border-accent"
      >
        {variantOptions.map((option) => (
          <option key={`${id}-${option.label}-${option.value}`} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>

      {hasNoVariants ? (
        <p id={helperTextId} className="mt-2 text-xs text-gray-400">
          No variants available
        </p>
      ) : null}
    </div>
  );
}
