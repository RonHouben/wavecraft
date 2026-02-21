/**
 * Processor component - displays all controls for a single processor.
 */

import React from 'react';
import { ParameterSlider } from './ParameterSlider';
import { ParameterSelect } from './ParameterSelect';
import { ParameterToggle } from './ParameterToggle';
import type { ParameterInfo } from './types';

export interface ProcessorParameter extends ParameterInfo {
  readonly onChange: (value: number | boolean) => void | Promise<void>;
  readonly disabled?: boolean;
}

export interface ProcessorProps {
  readonly id: string;
  readonly title?: string;
  readonly parameters: ProcessorParameter[];
}

export function Processor({ id, title, parameters }: Readonly<ProcessorProps>): React.JSX.Element {
  return (
    <div className="border-green-200! border-spacing-300 bg-blue-100">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-400">
        {title ?? id}
      </h3>

      <div className="space-y-3">
        {parameters.map((param) => {
          switch (param.type) {
            case 'bool':
              return (
                <ParameterToggle
                  key={param.id}
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
                  key={param.id}
                  id={param.id}
                  name={param.name}
                  value={typeof param.value === 'number' ? param.value : 0}
                  options={param.variants ?? []}
                  disabled={param.disabled}
                  onChange={param.onChange}
                />
              );
            case 'float':
              return (
                <ParameterSlider
                  key={param.id}
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
        })}
      </div>
    </div>
  );
}
