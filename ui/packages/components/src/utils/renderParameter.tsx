import React from 'react';

import type { ProcessorParameter } from '../types';
import { ParameterSelect } from '../ParameterSelect';
import { ParameterSlider } from '../ParameterSlider';
import { ParameterToggle } from '../ParameterToggle';

function toEnumOptions(variants?: readonly (string | undefined)[]): string[] {
  if (!variants || variants.length === 0) {
    return [];
  }

  return variants.filter((variant): variant is string => typeof variant === 'string');
}

export function renderParameter(param: ProcessorParameter, key: string): React.ReactNode {
  const enumOptions = toEnumOptions(param.variants);

  switch (param.type) {
    case 'bool':
      return (
        <ParameterToggle
          key={key}
          id={param.id}
          name={param.name}
          value={Boolean(param.value)}
          disabled={param.disabled}
          onChange={param.onChange}
        />
      );
    case 'enum':
      return (
        <ParameterSelect
          key={key}
          id={param.id}
          name={param.name}
          value={typeof param.value === 'number' ? param.value : 0}
          options={enumOptions}
          disabled={param.disabled}
          onChange={param.onChange}
        />
      );
    case 'float':
      return (
        <ParameterSlider
          key={key}
          id={param.id}
          name={param.name}
          value={typeof param.value === 'number' ? param.value : 0}
          min={param.min}
          max={param.max}
          unit={param.unit}
          disabled={param.disabled}
          onChange={param.onChange as (value: number) => void | Promise<void>}
        />
      );
    default:
      return null;
  }
}
