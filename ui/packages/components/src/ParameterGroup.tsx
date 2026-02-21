/**
 * ParameterGroup component - displays a group of parameters with a header.
 *
 * This component provides visual organization for parameters by grouping them
 * under a common header. It's typically used with the useParameterGroups hook.
 */

import React from 'react';
import type { ProcessorParameter } from './Processor';
import { parameterListClass, sectionHeadingClass } from './utils/classNames';
import { renderParameter } from './utils/renderParameter';

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
      <h3 className={sectionHeadingClass}>{group.name}</h3>

      {/* Parameter list */}
      <div className={parameterListClass}>
        {group.parameters.map((param) => renderParameter(param, param.id))}
      </div>
    </div>
  );
}
