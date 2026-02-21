/**
 * ParameterGroup component - displays a group of parameters with a header.
 *
 * This component provides visual organization for parameters by grouping them
 * under a common header. It's typically used with the useParameterGroups hook.
 */

import React from 'react';
import { ParameterSlider } from './ParameterSlider';
import { ParameterSelect } from './ParameterSelect';
import { ParameterToggle } from './ParameterToggle';
import type { ProcessorParameter } from './Processor';

type ParameterGroupType = {
  name: string;
  parameters: ProcessorParameter[];
};

export interface ParameterGroupProps {
  /** The parameter group to display */
  group: ParameterGroupType;
}

/**
 * Displays a group of parameters with a header and visual grouping.
 *
 * @example
 * ```tsx
 * const { parameters } = useAllParameters();
 * const groups = useParameterGroups(parameters);
 *
 * return (
 *   <div className="space-y-4">
 *     {groups.map(group => (
 *       <ParameterGroup key={group.name} group={group} />
 *     ))}
 *   </div>
 * );
 * ```
 */
export function ParameterGroup({ group }: Readonly<ParameterGroupProps>): React.JSX.Element {
  return (
    <div className="space-y-2">
      {/* Group header */}
      <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-400">{group.name}</h3>

      {/* Parameter list */}
      <div className="space-y-3">
        {group.parameters.map((param) =>
          param.type === 'bool' ? (
            <ParameterToggle
              key={param.id}
              id={param.id}
              name={param.name}
              value={Boolean(param.value)}
              disabled={param.disabled}
              onChange={param.onChange}
            />
          ) : param.type === 'enum' ? (
            <ParameterSelect
              key={param.id}
              id={param.id}
              name={param.name}
              value={typeof param.value === 'number' ? param.value : 0}
              options={param.variants ?? []}
              disabled={param.disabled}
              onChange={param.onChange as (value: number) => void | Promise<void>}
            />
          ) : (
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
          )
        )}
      </div>
    </div>
  );
}
