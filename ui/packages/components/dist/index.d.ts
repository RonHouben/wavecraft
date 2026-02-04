import { default as default_2 } from 'react';
import { ParameterGroup as ParameterGroup_2 } from '../../core/src';

export declare function ConnectionStatus(): default_2.JSX.Element;

export declare function LatencyMonitor(): default_2.JSX.Element;

export declare function Meter(): default_2.JSX.Element;

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
export declare function ParameterGroup({ group }: Readonly<ParameterGroupProps>): default_2.JSX.Element;

declare interface ParameterGroupProps {
    /** The parameter group to display */
    group: ParameterGroup_2;
}

export declare function ParameterSlider({ id }: ParameterSliderProps): default_2.JSX.Element;

declare interface ParameterSliderProps {
    readonly id: string;
}

export declare function ParameterToggle({ id }: ParameterToggleProps): default_2.JSX.Element;

declare interface ParameterToggleProps {
    readonly id: string;
}

export declare function ResizeControls(): default_2.JSX.Element;

export declare function ResizeHandle(): default_2.JSX.Element;

export declare function VersionBadge(): default_2.JSX.Element;

export { }
