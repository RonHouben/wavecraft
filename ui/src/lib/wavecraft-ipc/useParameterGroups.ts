/**
 * Hook for organizing parameters into groups based on their group metadata.
 *
 * This hook takes an array of parameters and organizes them into groups
 * for better UI organization. Parameters without a group are placed in
 * a default "Parameters" group.
 */

import { useMemo } from 'react';
import type { ParameterInfo } from './types';

export interface ParameterGroup {
  name: string;
  parameters: ParameterInfo[];
}

/**
 * Organize parameters into groups based on their group metadata.
 *
 * @param parameters - Array of all parameters
 * @returns Array of parameter groups, each containing parameters for that group
 *
 * @example
 * ```tsx
 * const { parameters } = useAllParameters();
 * const groups = useParameterGroups(parameters);
 *
 * return (
 *   <>
 *     {groups.map(group => (
 *       <ParameterGroup key={group.name} group={group} />
 *     ))}
 *   </>
 * );
 * ```
 */
export function useParameterGroups(parameters: ParameterInfo[]): ParameterGroup[] {
  return useMemo(() => {
    // Group parameters by their group field
    const grouped = new Map<string, ParameterInfo[]>();

    for (const param of parameters) {
      const groupName = param.group ?? 'Parameters';
      const existing = grouped.get(groupName) ?? [];
      existing.push(param);
      grouped.set(groupName, existing);
    }

    // Convert map to array of groups, sorted by group name
    // Exception: "Parameters" (default group) always comes first
    const groups: ParameterGroup[] = Array.from(grouped.entries())
      .map(([name, parameters]) => ({ name, parameters }))
      .sort((a, b) => {
        if (a.name === 'Parameters') return -1;
        if (b.name === 'Parameters') return 1;
        return a.name.localeCompare(b.name);
      });

    return groups;
  }, [parameters]);
}
