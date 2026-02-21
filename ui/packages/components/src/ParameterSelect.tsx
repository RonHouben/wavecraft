/**
 * ParameterSelect - Dropdown control for enum parameters.
 */

import React from 'react';
import { focusRingClass, interactionStateClass } from './utils/classNames';

export interface ParameterSelectProps {
  readonly id: string;
  readonly name: string;
  readonly value: number;
  readonly options: string[];
  readonly disabled?: boolean;
  readonly onChange: (value: number) => void | Promise<void>;
}

export function ParameterSelect({
  id,
  name,
  value,
  options,
  disabled = false,
  onChange,
}: Readonly<ParameterSelectProps>): React.JSX.Element {
  const variantOptions = options.map((label, index) => ({ value: index, label }));
  const hasNoVariants = variantOptions.length === 0;
  const helperTextId = hasNoVariants ? `select-${id}-helper` : undefined;

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>): void => {
    const nextValue = Number(event.currentTarget.value);
    void onChange(nextValue);
  };

  return (
    <div className="mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <div className="mb-2 flex items-center justify-between">
        <label htmlFor={`select-${id}`} className="font-semibold text-gray-200">
          {name}
        </label>
      </div>

      <select
        id={`select-${id}`}
        value={value}
        onChange={handleChange}
        disabled={disabled || hasNoVariants}
        aria-describedby={helperTextId}
        className={`w-full rounded border border-plugin-border bg-plugin-dark px-3 py-2 text-sm text-gray-200 focus:border-accent ${focusRingClass} ${interactionStateClass}`}
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
