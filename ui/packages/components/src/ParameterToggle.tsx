/**
 * ParameterToggle - Toggle control for boolean parameters
 */

import React from 'react';
import { focusRingClass, interactionStateClass } from './utils/classNames';

export interface ParameterToggleProps {
  readonly id: string;
  readonly name: string;
  readonly value: boolean;
  readonly disabled?: boolean;
  readonly onChange: (value: boolean) => void | Promise<void>;
}

export function ParameterToggle({
  id,
  name,
  value,
  disabled = false,
  onChange,
}: Readonly<ParameterToggleProps>): React.JSX.Element {
  const isOn = Boolean(value);

  const handleToggle = (): void => {
    const newValue = !isOn;
    void onChange(newValue);
  };

  return (
    <div className="mb-4 flex items-center justify-between rounded-lg border border-plugin-border bg-plugin-surface p-4">
      <label htmlFor={`toggle-${id}`} className="font-semibold text-gray-200">
        {name}
      </label>
      <button
        id={`toggle-${id}`}
        className={`relative h-[26px] w-[50px] cursor-pointer rounded-full border-none motion-safe:transition-colors motion-safe:duration-200 ${focusRingClass} ${interactionStateClass} ${
          isOn ? 'bg-accent hover:bg-accent-light' : 'bg-gray-600 hover:bg-gray-500'
        }`}
        onClick={handleToggle}
        aria-pressed={isOn}
        disabled={disabled}
      >
        <span
          className={`absolute top-[3px] h-5 w-5 rounded-full bg-white motion-safe:transition-[left] motion-safe:duration-200 ${
            isOn ? 'left-[27px]' : 'left-[3px]'
          }`}
        ></span>
      </button>
    </div>
  );
}
