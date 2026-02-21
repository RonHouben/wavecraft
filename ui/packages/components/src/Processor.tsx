/**
 * Processor component - displays all controls for a single processor.
 */

import React from 'react';
import type { ParameterInfo } from './types';
import { parameterListClass, sectionHeadingClass, surfaceCardClass } from './utils/classNames';
import { renderParameter } from './utils/renderParameter';

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
    <div className={`space-y-2 ${surfaceCardClass}`}>
      <h3 className={sectionHeadingClass}>{title ?? id}</h3>

      <div className={parameterListClass}>
        {parameters.map((param) => renderParameter(param, param.id))}
      </div>
    </div>
  );
}
